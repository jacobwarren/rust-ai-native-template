"""Explicit paired holdout evaluation. No optimization or candidate edits occur here."""
from pathlib import Path
import argparse
import json
from experiments.evaluator.spec import Suite, digest, positive
from experiments.evaluator.trial import evaluate, save
from .config import task_key


def compare(selection_path, suite_path, output, repeats=3, max_trials=100):
    selection_path = Path(selection_path).resolve()
    selection = json.loads(selection_path.read_text())
    suite = Suite.load(suite_path)
    if {t.id for t in suite.tasks} & set(selection["search_task_ids"]) or {task_key(t) for t in suite.tasks} & set(selection["search_task_keys"]):
        raise ValueError("holdout overlaps optimization tasks")
    if suite.identity != selection["agent_identity"] or list(suite.agent.argv) != selection["agent_argv"] or suite.agent.timeout_seconds != selection["agent_timeout_seconds"]:
        raise ValueError("agent changed between search and final comparison")
    if 2 * len(suite.tasks) * positive(repeats) > positive(max_trials):
        raise ValueError("paired comparison exceeds max_trials")
    candidates = {name: selection_path.with_name(name + "-skill.md").read_text() for name in ("seed", "selected")}
    for name, candidate in candidates.items():
        if digest(candidate.encode()) != selection[name + "_sha256"]:
            raise ValueError("selected/seed skill changed since optimization")
    output = Path(output).resolve()
    output.mkdir(parents=True, exist_ok=False)
    report = {"schema_version": 1, "complete": False, "suite_sha256": suite.source_sha256,
              "reports": [], "promotion": "review required; no files are promoted automatically"}
    save(output / "comparison.json", report)
    totals = {"seed": 0, "selected": 0}
    for repeat in range(repeats):
        # Counterbalance order to reduce systematic warm-up/order effects.
        order = ("seed", "selected") if repeat % 2 == 0 else ("selected", "seed")
        for task in suite.tasks:
            for name in order:
                trial = evaluate(suite, task, candidates[name], output / "trials")
                totals[name] += trial["status"] == "accepted"
                report["reports"].append({"candidate": name, "repeat": repeat, **trial})
                save(output / "comparison.json", report)
                if trial["status"] not in {"accepted", "rejected", "policy_violation", "timeout"}:
                    raise RuntimeError(f"final evaluation failed: {trial['report_path']}")
    count = repeats * len(suite.tasks)
    report.update(complete=True, trials_per_candidate=count, accepted=totals,
                  acceptance_delta=(totals["selected"] - totals["seed"]) / count)
    save(output / "comparison.json", report)
    return report


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--selection", type=Path, required=True)
    parser.add_argument("--suite", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--repeats", type=int, default=3)
    parser.add_argument("--max-trials", type=int, default=100)
    args = parser.parse_args()
    result = compare(args.selection, args.suite, args.output, args.repeats, args.max_trials)
    print(json.dumps({k: v for k, v in result.items() if k != "reports"}, indent=2))


if __name__ == "__main__":
    main()
