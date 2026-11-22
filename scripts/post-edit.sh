#!/usr/bin/env bash
# Claude Code PostToolUse hook: after an edit to a crate, type-check that crate
# and check its formatting, and hand any failure straight back to the agent.
# Reads the hook event on stdin. Exit 2 shows stderr to the agent; exit 0 is silent.
set -u
root=$(cd "$(dirname "$0")/.." && pwd)
event=$(head -c 1048576)
file=$(printf '%s' "$event" | sed -n 's/.*"file_path"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/p' | head -n 1)
case "$file" in "$root"/crates/*/*) ;; crates/*/*) file="$root/$file" ;; *) exit 0 ;; esac
case "$file" in *.rs | */Cargo.toml) ;; *) exit 0 ;; esac
crate=${file#"$root"/crates/}; crate=${crate%%/*}
package=$(sed -n 's/^name[[:space:]]*=[[:space:]]*"\(.*\)"/\1/p' "$root/crates/$crate/Cargo.toml" | head -n 1)
[ -n "$package" ] || exit 0
cd "$root" || exit 0
[ -n "${POST_EDIT_LOG:-}" ] && echo "$package" >> "$POST_EDIT_LOG"
if ! out=$(cargo check --quiet --locked --all-targets -p "$package" --message-format short 2>&1); then
  { echo "cargo check -p $package failed after this edit:"; printf '%s\n' "$out" | tail -n 40; } >&2
  exit 2
fi
if ! cargo fmt -p "$package" -- --check >/dev/null 2>&1; then
  echo "$package is not formatted: run cargo fmt -p $package" >&2
  exit 2
fi
exit 0
