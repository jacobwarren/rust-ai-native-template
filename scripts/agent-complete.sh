#!/usr/bin/env bash
# Opt-in stop/completion hook. Map exit 2 to a blocking diagnostic in your host.
set -euo pipefail
if [[ -z "${HARNESS_TASK_ID:-}" || -z "${HARNESS_BASE:-}" ]]; then
  echo 'Set HARNESS_TASK_ID and HARNESS_BASE before using the completion hook.' >&2
  exit 2
fi
cd "$(dirname "$0")/.."
if cargo xtask agent-stop --task "$HARNESS_TASK_ID" --base "$HARNESS_BASE" \
  --report "${HARNESS_REPORT:-target/verification/ci.json}" >&2; then
  printf '{}\n'
  exit 0
else
  echo 'Completion gate failed. Answer the task-review questions and fix the reported items, or record an honest partial/blocked handoff. No new gap is required when none was found. Do not claim completion.' >&2
  exit 2
fi
