"""Bounded Unix process groups with file-backed output; never invoke a shell."""
from pathlib import Path
import json
import os
import signal
import subprocess
import time


def run(command, cwd, artifacts, label, request=None):
    if os.name != "posix":
        raise RuntimeError("evaluation requires Linux or macOS")
    artifacts = Path(artifacts)
    artifacts.mkdir(parents=True, exist_ok=True)
    out, err = artifacts / f"{label}.stdout", artifacts / f"{label}.stderr"
    started = time.monotonic()
    result = {"argv": list(command.argv), "status": "error", "exit_code": None,
              "stdout": str(out), "stderr": str(err), "duration_seconds": 0.0}
    state = artifacts / "state" / label
    state.mkdir(parents=True, exist_ok=True)
    env = os.environ.copy()
    env.update(CARGO_TARGET_DIR=str(Path(cwd) / "target"), PYTHONDONTWRITEBYTECODE="1",
               HARNESS_RUN_ID=artifacts.name, APP_DATA_DIR=str(state))
    # Preserve the normal harness's own outer process group behavior.
    env.pop("HARNESS_IN_PROCESS_GROUP", None)
    inp = artifacts / f"{label}.stdin"
    inp.write_text(json.dumps(request) if request is not None else "")
    try:
        with inp.open("rb") as stdin, out.open("wb") as stdout, err.open("wb") as stderr:
            child = subprocess.Popen(command.argv, cwd=cwd, env=env, stdin=stdin,
                                     stdout=stdout, stderr=stderr, start_new_session=True)
            try:
                code = child.wait(timeout=command.timeout_seconds)
                result.update(status="passed" if code == 0 else "failed", exit_code=code)
            except subprocess.TimeoutExpired:
                result["status"] = "timeout"
            finally:
                try:
                    os.killpg(child.pid, signal.SIGKILL)
                except ProcessLookupError:
                    pass
                child.wait()
    except OSError as error:
        result["error"] = str(error)
    result["duration_seconds"] = time.monotonic() - started
    return result


def json_output(outcome, limit=1_000_000):
    path = Path(outcome["stdout"])
    if path.stat().st_size > limit:
        raise ValueError("protocol response exceeds size limit")
    return json.loads(path.read_text())
