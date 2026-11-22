#!/usr/bin/env bash
# The whole gate, the same locally and in CI. Run it before you hand work off.
set -euo pipefail
cd "$(dirname "$0")/.."
step() { printf '\n== %s\n' "$*"; "$@"; }
step cargo fmt --all -- --check
step cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
step cargo test --workspace --locked
RUSTDOCFLAGS="-D warnings" step cargo doc --workspace --no-deps --locked
printf '\nci: all checks passed\n'
