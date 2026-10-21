"""Deterministic plumbing fixture. Never use this as an agent-quality benchmark."""
import json
from pathlib import Path
import sys

request = json.load(sys.stdin)
mode = sys.argv[1]
root = Path(request["workspace"])
if mode in {"fix", "forbidden"}:
    path = root / "src/main.rs"
    path.write_text(path.read_text().replace(".unwrap_or_default()", ".unwrap_or_default().to_uppercase()"))
if mode == "forbidden":
    (root / "Cargo.toml").write_text("# unauthorized policy edit\n")
print(json.dumps({"schema_version": 1, "claimed_complete": True,
                  "summary": f"Scripted {mode} fixture; this claim is not a grade."}))
