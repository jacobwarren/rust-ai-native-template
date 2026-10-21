"""Create a disposable real Rust task; scripted agents only test evaluator plumbing."""
from pathlib import Path
import argparse
import json
import sys
from .workspace import git, initialize_git
from .spec import Suite
from .trial import evaluate, save

GRADER = '''from pathlib import Path
import json, subprocess
failures = []
for name, expected in [("Ada", "Hello, ADA!"), ("Zoë", "Hello, ZOË!"), ("aLiCe", "Hello, ALICE!")]:
    run = subprocess.run(["cargo", "run", "--quiet", "--locked", "--offline", "--", name], capture_output=True, text=True, timeout=60)
    if run.returncode != 0 or run.stdout.strip() != expected:
        failures.append({"input": name, "actual": run.stdout.strip(), "exit": run.returncode})
print(json.dumps({"accepted": not failures, "reason": json.dumps(failures) if failures else "all independent cases passed"}))
raise SystemExit(1 if failures else 0)
'''


def create_demo(destination):
    destination = Path(destination).resolve()
    destination.mkdir(parents=True, exist_ok=False)
    repo = destination / "repo"
    (repo / "src").mkdir(parents=True)
    (repo / "Cargo.toml").write_text('[package]\nname="greeting-task"\nversion="0.1.0"\nedition="2024"\n')
    (repo / "Cargo.lock").write_text('version = 4\n[[package]]\nname="greeting-task"\nversion="0.1.0"\n')
    (repo / ".gitignore").write_text('/target/\n')
    (repo / "src/main.rs").write_text('fn main() { let name = std::env::args().nth(1).unwrap_or_default(); println!("Hello, {name}!"); }\n')
    initialize_git(repo)
    commit = git(repo, "rev-parse", "HEAD").decode().strip()
    grader = destination / "grader.py"
    grader.write_text(GRADER)
    adapter = Path(__file__).with_name("demo_agent.py")
    config = {"schema_version": 1, "agent_identity": "scripted-fixture-not-a-model",
              "agent": {"argv": [sys.executable, str(adapter), "noop"], "timeout_seconds": 10},
              "tasks": [{"id": "uppercase-name", "repo": str(repo), "commit": commit,
                         "prompt": "Uppercase the entire supplied name using Unicode case conversion. Preserve the Hello, NAME! output format.",
                         "editable_files": ["src/main.rs"],
                         "regression": [{"argv": ["cargo", "test", "--locked", "--offline"], "timeout_seconds": 120}],
                         "grader": str(grader), "grader_timeout_seconds": 120}]}
    path = destination / "suite.json"
    save(path, config)
    return path, config


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", required=True, type=Path)
    args = parser.parse_args()
    path, config = create_demo(args.output)
    cases = []
    for mode, expected in [("noop", "rejected"), ("fix", "accepted"), ("forbidden", "policy_violation")]:
        config["agent"]["argv"][-1] = mode
        save(path, config)
        suite = Suite.load(path)
        report = evaluate(suite, suite.tasks[0], "Inspect the input boundary and preserve output contracts.", args.output / "trials")
        cases.append({"mode": mode, "expected": expected, "actual": report["status"], "report_path": report["report_path"]})
    summary = {"ok": all(c["actual"] == c["expected"] for c in cases), "cases": cases,
               "scope": "scripted evaluator verification; no model or GEPA performance measurement"}
    save(args.output / "demo-summary.json", summary)
    print(json.dumps(summary, indent=2))
    return 0 if summary["ok"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
