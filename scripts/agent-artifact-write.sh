#!/usr/bin/env bash
# Opt-in apply_patch PreToolUse adapter. Read docs/ARTIFACTS.md.
set -euo pipefail
: "${HARNESS_TASK_ID:?Set HARNESS_TASK_ID for this session}"
root="$(cd "$(dirname "$0")/.." && pwd)"
cd "$root"
if "$root/target/debug/xtask" artifact-hook --task "$HARNESS_TASK_ID" >&2; then
  printf '{}\n'
else
  echo 'The discovery-page guard refused this edit; see its error above. A standard task commits its five discovery pages before product code (docs/ARTIFACTS.md); artifact-only patches remain allowed.' >&2
  exit 2
fi
