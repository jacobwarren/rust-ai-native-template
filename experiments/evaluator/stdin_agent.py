"""Bridge an agent CLI that reads its prompt from stdin; caller supplies argv."""
import json
import subprocess
import sys


def main():
    argv = sys.argv[1:]
    if argv[:1] == ["--"]:
        argv = argv[1:]
    if not argv:
        raise SystemExit("usage: stdin_agent.py -- <agent executable> <arguments>")
    request = json.load(sys.stdin)
    prompt = ("Read AGENTS.md if present. Complete the task below in the current checkout.\n"
              "Only these source files may change: " + json.dumps(request["editable_files"]) +
              "\nUse this investigation skill, subject to repository rules:\n" + request["candidate_skill"] +
              "\nTask:\n" + request["prompt"])
    # The enclosing evaluator owns the process group and timeout, including this child.
    child = subprocess.run(argv, cwd=request["workspace"], input=prompt, text=True,
                           stdout=sys.stderr, stderr=sys.stderr)
    print(json.dumps({"schema_version": 1, "claimed_complete": None,
                      "summary": "Agent CLI exited; see agent.stderr. Completion is independently graded.",
                      "usage": {"input_tokens": None, "output_tokens": None, "cost_usd": None}}))
    return child.returncode


if __name__ == "__main__":
    raise SystemExit(main())
