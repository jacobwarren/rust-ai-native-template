#!/usr/bin/env bash
# Opt-in apply_patch PreToolUse adapter. Read docs/ARTIFACTS.md.
set -euo pipefail
: "${HARNESS_TASK_ID:?Set HARNESS_TASK_ID for this session}"
root="$(cd "$(dirname "$0")/.." && pwd)"
cd "$root"
if "$root/target/debug/xtask" artifact-hook --task "$HARNESS_TASK_ID" >&2; then
  printf '{}\n'
else
  echo 'Write and commit the five discovery pages (docs/ARTIFACTS.md) before editing product code. Artifact-only patches remain allowed.' >&2
  exit 2
fi
