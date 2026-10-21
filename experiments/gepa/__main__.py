from pathlib import Path
import argparse
import contextlib
import json
from .config import Experiment
from .optimize import optimize


def main():
    parser = argparse.ArgumentParser(description="Opt-in GEPA skill search; may invoke paid configured models")
    parser.add_argument("--config", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True, help="new directory outside task repos")
    args = parser.parse_args()
    experiment = Experiment.load(args.config)
    args.output.parent.mkdir(parents=True, exist_ok=True)
    log = args.output.with_name(args.output.name + ".log")
    with log.open("x") as stream, contextlib.redirect_stdout(stream), contextlib.redirect_stderr(stream):
        result = optimize(experiment, args.output)
    print(json.dumps({"complete": result["complete"], "selection": result["selection"], "log": str(log)}, indent=2))


if __name__ == "__main__":
    main()
