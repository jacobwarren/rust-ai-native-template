"""A trial scores independent acceptance, never a completion claim or CI alone."""
from pathlib import Path
import json
import math
import shutil
import sys
import tempfile
import time
from .process import json_output, run
from .spec import Command, digest, object_fields
from .workspace import changed_files, initialize_git, inventory, snapshot, tree_digest


def save(path, report):
    temporary = path.with_suffix(".tmp")
    temporary.write_text(json.dumps(report, indent=2, allow_nan=False) + "\n")
    temporary.replace(path)


def grade(task, workspace, artifacts, script, label):
    result = run(Command((sys.executable, str(script)), task.grader_timeout_seconds),
                 workspace, artifacts, label)
    if result["status"] not in {"passed", "failed"}:
        return {"valid": False, "process": result}
    try:
        response = object_fields(json_output(result), ["accepted", "reason"])
        if type(response["accepted"]) is not bool or not isinstance(response["reason"], str):
            raise ValueError("invalid grader response types")
        if result["exit_code"] != (0 if response["accepted"] else 1):
            raise ValueError("grader exit code disagrees with acceptance")
        return {"valid": True, **response, "process": result}
    except (ValueError, OSError) as error:
        return {"valid": False, "error": str(error), "process": result}


def agent_response(outcome):
    raw = object_fields(json_output(outcome), ["schema_version", "claimed_complete", "summary"], ["usage"])
    if type(raw["schema_version"]) is not int or raw["schema_version"] != 1:
        raise ValueError("unsupported agent protocol")
    if raw["claimed_complete"] is not None and type(raw["claimed_complete"]) is not bool:
        raise ValueError("claimed_complete must be boolean or null")
    if not isinstance(raw["summary"], str):
        raise ValueError("summary must be text")
    usage = raw.get("usage", {})
    object_fields(usage, [], ["input_tokens", "output_tokens", "cost_usd"])
    normalized = {}
    for key in ("input_tokens", "output_tokens", "cost_usd"):
        value = usage.get(key)
        if value is not None:
            if type(value) not in {int, float} or not math.isfinite(value) or value < 0:
                raise ValueError("usage must be nonnegative finite numbers or null")
            if key != "cost_usd" and type(value) is not int:
                raise ValueError("token counts must be integers")
        normalized[key] = value
    return {**raw, "usage": normalized, "usage_source": "agent_reported"}


def evaluate(suite, task, candidate, output):
    output = Path(output).resolve()
    if output.is_relative_to(task.repo):
        raise ValueError("trial output must be outside the task repository")
    if not isinstance(candidate, str) or not candidate.strip() or len(candidate.encode()) > 32_768:
        raise ValueError("candidate skill must contain 1-32768 UTF-8 bytes")
    output.mkdir(parents=True, exist_ok=True)
    artifacts = Path(tempfile.mkdtemp(prefix=task.id + "-", dir=output))
    report_path = artifacts / "report.json"
    (artifacts / "candidate.md").write_text(candidate)
    # Copy the already-read grader bytes. Agent checkout never supplies a grader.
    script = artifacts / "grader.py"
    script.write_bytes(task.grader_bytes)
    report = {"schema_version": 1, "complete": False, "status": "incomplete", "score": None,
              "task_id": task.id, "commit": task.commit, "agent_identity": suite.identity,
              "suite_sha256": suite.source_sha256, "grader_sha256": digest(task.grader_bytes),
              "candidate_sha256": digest(candidate.encode()), "report_path": str(report_path),
              "duration_seconds": None, "agent": None, "human_effort_seconds": None}
    save(report_path, report)
    started = time.monotonic()
    try:
        with tempfile.TemporaryDirectory(prefix="agent-eval-") as temp:
            home = Path(temp)
            base, agent, grading = home / "base", home / "agent", home / "grading"
            snapshot(task.repo, task.commit, base)
            before = inventory(base)
            report["base_source_sha256"] = tree_digest(before)
            baseline = grade(task, base, artifacts, script, "baseline-acceptance")
            report["baseline_acceptance"] = baseline
            if not baseline["valid"]:
                report["status"] = "error"
                return report
            if baseline["accepted"]:
                report["status"] = "invalid_task"
                return report
            baseline_checks = [run(c, base, artifacts, f"baseline-regression-{i}")
                               for i, c in enumerate(task.regression)]
            report["baseline_regression"] = baseline_checks
            if any(c["status"] != "passed" for c in baseline_checks) or inventory(base) != before:
                report["status"] = "invalid_task"
                return report
            shutil.copytree(base, agent, ignore=lambda directory, names: ["target"] if Path(directory) == base else [])
            initialize_git(agent)
            request = {"schema_version": 1, "task_id": task.id, "workspace": str(agent),
                       "prompt": task.prompt, "candidate_skill": candidate,
                       "editable_files": list(task.editable_files)}
            outcome = run(suite.agent, agent, artifacts, "agent", request)
            report["agent_process"] = outcome
            save(report_path, report)
            if outcome["status"] != "passed":
                report["status"] = "timeout" if outcome["status"] == "timeout" else "error"
                return report
            try:
                report["agent"] = agent_response(outcome)
            except (ValueError, OSError) as error:
                report.update(status="protocol_error", error=str(error))
                return report
            try:
                after = inventory(agent)
            except ValueError as error:
                report.update(status="policy_violation", error=str(error))
                return report
            changed = changed_files(before, after)
            report["changed_files"] = changed
            forbidden = sorted(set(changed) - set(task.editable_files))
            if forbidden or inventory(base) != before or script.read_bytes() != task.grader_bytes:
                report.update(status="policy_violation", forbidden_files=forbidden)
                return report
            # Recreate grading state; discard agent .git, target, commands and logs.
            shutil.copytree(base, grading, ignore=lambda directory, names: ["target"] if Path(directory) == base else [])
            for name in changed:
                destination = grading / name
                if name not in after:
                    destination.unlink(missing_ok=True)
                else:
                    data = (agent / name).read_bytes()
                    if digest(data) != after[name][0]:
                        raise ValueError("agent source changed during patch capture")
                    destination.parent.mkdir(parents=True, exist_ok=True)
                    destination.write_bytes(data)
                    destination.chmod(0o755 if after[name][1] else 0o644)
                    evidence = artifacts / "changes" / name
                    evidence.parent.mkdir(parents=True, exist_ok=True)
                    evidence.write_bytes(data)
            grade_before = inventory(grading)
            report["graded_source_sha256"] = tree_digest(grade_before)
            report["regression"] = [run(c, grading, artifacts, f"regression-{i}")
                                    for i, c in enumerate(task.regression)]
            acceptance = grade(task, grading, artifacts, script, "acceptance")
            report["acceptance"] = acceptance
            if not acceptance["valid"] or inventory(grading) != grade_before:
                report["status"] = "error"
            elif any(c["status"] in {"error", "timeout"} for c in report["regression"]):
                report["status"] = "error"
            elif acceptance["accepted"] and all(c["status"] == "passed" for c in report["regression"]):
                report["status"] = "accepted"
            else:
                report["status"] = "rejected"
    except Exception as error:
        report.update(status="error", error=f"{type(error).__name__}: {error}")
    finally:
        report["complete"] = report["status"] != "incomplete"
        report["score"] = (1.0 if report["status"] == "accepted" else 0.0) if report["complete"] else None
        report["duration_seconds"] = time.monotonic() - started
        save(report_path, report)
    return report
