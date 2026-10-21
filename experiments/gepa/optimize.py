"""GEPA 0.1.4 adapter; holdouts are evaluated only by the separate finalize command."""
from pathlib import Path
import importlib
import importlib.metadata
import time
from experiments.evaluator.spec import digest
from experiments.evaluator.trial import evaluate, save
from .config import task_key

PIN = "0.1.4"


def optimize(experiment, output, reflection=None):
    # Delayed import keeps all ordinary development and evaluator tests GEPA-free.
    if importlib.metadata.version("gepa") != PIN:
        raise RuntimeError(f"this adapter is verified against gepa=={PIN}; install requirements.lock")
    oa = importlib.import_module("gepa.optimize_anything")
    output = Path(output).resolve()
    output.mkdir(parents=True, exist_ok=False)
    for task in experiment.train.tasks + experiment.validation.tasks:
        if output.is_relative_to(task.repo):
            raise ValueError("optimization artifacts must be outside task repositories")
    started = time.monotonic()
    state = {"schema_version": 1, "complete": False, "gepa_version": PIN,
             "experiment_sha256": experiment.sha256, "evaluations": [],
             "agent_identity": experiment.train.identity,
             "train_suite_sha256": experiment.train.source_sha256,
             "validation_suite_sha256": experiment.validation.source_sha256,
             "agent_reported_cost_usd": None, "gepa_reported_reflection_cost_usd": None,
             "reflection_backend": "configured_model" if reflection is None else "scripted_test",
             "scope": "Configured task distribution only; candidate requires independent final evaluation"}
    save(output / "optimization.json", state)
    lookup = {t.id: (suite, t) for suite in (experiment.train, experiment.validation) for t in suite.tasks}

    def evaluator(candidate, example):
        if len(state["evaluations"]) >= experiment.max_evals:
            raise RuntimeError("hard trial budget exhausted; no further agent call admitted")
        if time.monotonic() - started >= experiment.admission_seconds:
            raise RuntimeError("trial admission deadline exceeded")
        if not isinstance(candidate, str) or not candidate.strip() or len(candidate.encode()) > 32768:
            state["evaluations"].append({"status": "invalid_candidate", "score": 0.0})
            save(output / "optimization.json", state)
            return 0.0, {"reason": "skill must contain 1-32768 UTF-8 bytes"}
        suite, task = lookup[example]
        report = evaluate(suite, task, candidate, output / "trials")
        state["evaluations"].append(report)
        save(output / "optimization.json", state)
        if report["status"] not in {"accepted", "rejected", "policy_violation", "timeout"}:
            raise RuntimeError(f"evaluation infrastructure/protocol failure: {report['report_path']}")
        info = {"task_id": task.id, "status": report["status"],
                "reason": report.get("acceptance", {}).get("reason", report.get("error", "")),
                "duration_seconds": report["duration_seconds"],
                "forbidden_files": report.get("forbidden_files", []),
                "regression": [{"status": c["status"], "exit_code": c["exit_code"]}
                               for c in report.get("regression", [])]}
        return report["score"], info

    config = oa.GEPAConfig(
        engine=oa.EngineConfig(run_dir=str(output / "gepa"), max_metric_calls=experiment.max_evals,
            max_candidate_proposals=experiment.max_proposals,
            max_reflection_cost=experiment.max_reflection_cost_usd,
            parallel=False, cache_evaluation=False, use_cloudpickle=False, raise_on_exception=True),
        reflection=oa.ReflectionConfig(reflection_lm=reflection or experiment.model,
            reflection_minibatch_size=1, skip_perfect_score=True, perfect_score=1.0,
            reflection_lm_kwargs={"timeout": 60, "num_retries": 0}),
    )
    try:
        result = oa.optimize_anything(seed_candidate=experiment.seed, evaluator=evaluator,
            dataset=[t.id for t in experiment.train.tasks], valset=[t.id for t in experiment.validation.tasks],
            objective="Improve independent Rust task acceptance by refining one concise investigation skill.",
            background="Keep repository authority and acceptance criteria fixed. The skill is advisory. Never propose bypassing checks or reading hidden graders.",
            config=config)
        selected = result.best_candidate
        if not isinstance(selected, str):
            raise TypeError("expected a single textual skill")
        (output / "selected-skill.md").write_text(selected)
        (output / "seed-skill.md").write_text(experiment.seed)
        selection = {"schema_version": 1, "seed_sha256": digest(experiment.seed.encode()),
                     "selected_sha256": digest(selected.encode()), "best_validation_score": result.val_aggregate_scores[result.best_idx],
                     "search_task_ids": sorted(lookup),
                     "search_task_keys": sorted(task_key(task) for _, task in lookup.values()),
                     "agent_identity": experiment.train.identity,
                     "agent_argv": list(experiment.train.agent.argv),
                     "agent_timeout_seconds": experiment.train.agent.timeout_seconds,
                     "promotion": "pending independent holdout comparison and review"}
        save(output / "selection.json", selection)
        state.update(complete=True, selection=str(output / "selection.json"))
        return state
    except Exception as error:
        state["error"] = f"{type(error).__name__}: {error}"
        raise
    finally:
        state["duration_seconds"] = time.monotonic() - started
        state["gepa_reported_reflection_cost_usd"] = getattr(config.reflection.reflection_lm, "total_cost", None)
        costs = [r.get("agent", {}).get("usage", {}).get("cost_usd") if r.get("agent") else None
                 for r in state["evaluations"]]
        if costs and all(c is not None for c in costs):
            state["agent_reported_cost_usd"] = sum(costs)
        save(output / "optimization.json", state)
