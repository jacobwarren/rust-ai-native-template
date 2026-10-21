from pathlib import Path
import argparse
import json
from .spec import Suite, positive
from .trial import evaluate, save


def main():
    parser = argparse.ArgumentParser(description="Evaluate a skill using fixed tasks and independent graders")
    parser.add_argument("--suite", required=True, type=Path)
    parser.add_argument("--candidate", required=True, type=Path)
    parser.add_argument("--output", required=True, type=Path)
    parser.add_argument("--repeats", default=1, type=int)
    args = parser.parse_args()
    suite = Suite.load(args.suite)
    candidate = args.candidate.read_text()
    reports = [evaluate(suite, task, candidate, args.output)
               for _ in range(positive(args.repeats)) for task in suite.tasks]
    summary = {"schema_version": 1, "trials": len(reports),
               "accepted": sum(r["status"] == "accepted" for r in reports),
               "statuses": [r["status"] for r in reports],
               "reports": [r["report_path"] for r in reports]}
    save(args.output / "summary.json", summary)
    print(json.dumps(summary, indent=2))
    return 0 if summary["accepted"] == len(reports) else 1


if __name__ == "__main__":
    raise SystemExit(main())
