#!/usr/bin/env bash
# Default Stop hook for Claude Code and Codex: gate completion claims only.
# A turn that claims nothing stops. When a task record changed on this branch
# says "status": "complete", the existing completion gate judges the claim and
# a refusal keeps the agent working. Setting the record back to partial, with
# its remaining actions, always clears the block. Partial handoffs, stray files
# and policy receipts are checked by PR CI (cargo xtask handoff), not here.
#   HARNESS_BASE    base ref (default origin/main)
#   HARNESS_REPORT  CI evidence (default target/verification/ci.json)
#   HARNESS_XTASK   tests only: replaces `cargo xtask`
# No -E: there are no functions, and bash 3.2 would run the ERR trap inside the
# guarded $(...) calls below.
set -u
trap 'echo "agent-complete: internal error at line $LINENO. Tell the owner the stop gate needs a fix." >&2; exit 2' ERR

event=""
[ -t 0 ] || event=$(head -c 1048576) || event=""
cwd=$(printf '%s' "$event" | sed -n 's/.*"cwd"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/p' | head -n 1)
[ -n "$cwd" ] || cwd=$PWD
root=$(git -C "$cwd" rev-parse --show-toplevel 2>/dev/null) || exit 0
cd "$root"

base_ref=${HARNESS_BASE:-origin/main}
if ! base=$(git merge-base HEAD "$base_ref" 2>/dev/null); then
  echo "agent-complete: no merge base with $base_ref, so completion claims are not gated this turn." >&2
  exit 0
fi

ids=$({
  git diff --name-only "$base" -- harness/tasks harness/task-artifacts
  git ls-files -o --exclude-standard -- harness/tasks harness/task-artifacts
} | sed -n -e 's|^harness/tasks/\([^/]*\)\.json$|\1|p' -e 's|^harness/task-artifacts/\([^/]*\)/.*|\1|p' | sort -u)

# Must match how task_record reads the top-level status; a unit test checks it.
CLAIM='"status"[[:space:]]*:[[:space:]]*"complete"'
claims=""
for id in $ids; do
  if [ -f "harness/tasks/$id.json" ] && grep -Eq "$CLAIM" "harness/tasks/$id.json"; then
    claims="$claims $id"
  fi
done
[ -n "$claims" ] || exit 0

report=${HARNESS_REPORT:-target/verification/ci.json}
xtask=${HARNESS_XTASK:-cargo xtask}
for id in $claims; do
  out=$($xtask complete --task "$id" --base "$base" --report "$report" 2>&1) && code=0 || code=$?
  [ "$code" -ne 0 ] || continue
  if [ "$code" -eq 1 ] && [ "$(printf '%s\n' "$out" | head -n 1)" = "{" ]; then
    {
      echo "Completion claim refused: harness/tasks/$id.json says complete, but the completion gate failed:"
      printf '%s\n' "$out" | head -c 8000
      echo
      echo "Fix what it reports (run cargo xtask check ci > $report for fresh evidence), or set the record back to partial with its remaining actions. Do not claim completion."
    } >&2
  else
    {
      echo "Completion claim not checked: harness/tasks/$id.json says complete, but the completion gate could not run (exit $code):"
      printf '%s\n' "$out" | tail -n 20
      echo "Fix the build first (cargo check --locked says why), or set the record back to partial with its remaining actions."
    } >&2
  fi
  exit 2
done
exit 0
