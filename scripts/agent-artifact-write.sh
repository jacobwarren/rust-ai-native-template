#!/usr/bin/env bash
# Opt-in apply_patch PreToolUse/PostToolUse adapter. Read docs/ARTIFACTS.md.
set -euo pipefail
: "${HARNESS_TASK_ID:?Set HARNESS_TASK_ID for this session}"
root="$(cd "$(dirname "$0")/.." && pwd)"
cd "$root"
if "$root/target/debug/xtask" artifact-hook --task "$HARNESS_TASK_ID" --point "${1:-before-edit}" >&2; then
  printf '{}\n'
else
  echo 'Artifact checkpoint gate blocked this boundary. Read the diagnostic; update and register implementation notes before continuing. Artifact-only patches remain allowed.' >&2
  exit 2
fi
