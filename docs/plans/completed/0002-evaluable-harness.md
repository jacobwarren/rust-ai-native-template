# Make the harness measurable and restartable

Owner: maintainers. Status: completed. Starting point: delivered v1 archive, with
14 passing Rust tests and six CLI scenarios; no Git history was supplied.

## Purpose and observable outcome

Preserve normal Rust development and add an opt-in experiment path that evaluates
whether a coding agent completes a task. A no-op agent must fail an unmet task
even if repository CI passes. A deterministic fixture agent must pass the same
task by making the required change. GEPA can propose a bounded investigation
skill; acceptance criteria and the grader remain fixed.

## Progress

- [x] Read scoped instructions, supplied guidance, and current implementation.
- [x] Run baseline doctor and fast checks from the workspace root.
- [x] Introduce private parsed command specs and typed execution statuses.
- [x] Improve navigation, plan lifecycle, and semantic maintenance guidance.
- [x] Implement fresh-checkout evaluation, controlled edits, and trusted grading.
- [x] Add the optional GEPA adapter and test with the actual pinned package.
- [x] Verify all surfaces, record limits, and replace the delivered archive.

## Context and orientation

The product is an example greeting CLI. crates/domain contains Name and greet;
crates/app handles JSON process I/O. xtask/src/config.rs owns command admission;
runner.rs executes checks; scenarios.rs grades process behavior. Rust CI is
cargo xtask check ci. Optional Python experiments will live under experiments/,
with no Python or model dependency in the Rust build. Python standard-library
tests run separately. The task evaluator will reconstruct a grading tree from a
fixed Git commit and only permitted agent edits, avoiding agent-edited checks.

## Milestones and acceptance

First, preserve existing command JSON while rejecting empty executables, invalid
IDs, zero timeouts, and unknown fields at parsing time. Run cargo test --workspace
--locked and expect all existing and new command-boundary tests to pass.

Second, implement an evaluator with an explicit agent stdin protocol and fixed
external grader. The agent receives only the task, candidate skill, and checkout
location. Prove no-op failure, successful change, forbidden-edit rejection,
source-checkout preservation, timeout, and separate report statuses. Build a
small disposable Rust task repo for the executable demonstration.

Third, connect the pinned GEPA Python API to the same evaluator. Use train and
validation sets during search; invoke final holdouts explicitly after selection.
No credentials or paid runs are required for local contract tests. Real agent
and model performance remains unmeasured until the owner supplies them.

## Decisions

- Use an optional Python standard-library evaluator plus GEPA adapter so normal
  Cargo commands remain dependency-light. Keep the core harness in Rust.
- Freeze the task/agent configuration for each experiment and reconstruct the
  grading tree independently. Same-user process separation is not a security
  sandbox; malicious-agent evaluations require an external isolation boundary.
- Optimize one investigation skill initially. No autonomous policy weakening,
  task-grader edits, or automatic promotion to the canonical skill.

## Discoveries

The existing command boundary checked Vec length and timeout at runtime but did
not retain those guarantees in the type. Outcome status was a String. These were
replaced while preserving existing status strings in the report format.

The installed GEPA 0.1.4 API differs from current upstream documentation. The
adapter uses the verified GEPAConfig/EngineConfig/ReflectionConfig API and checks
the package version. A real-package integration test uses a scripted reflection
function; no provider calls are necessary to verify search and finalization.

## Recovery and idempotence

Repeated checks create unique artifact directories. Evaluation clones are
throwaway and never alter the source Git checkout. Reports survive failed tasks;
large workspaces are removed after evidence is captured. Interrupted experiments
are recorded as incomplete and are not reused as completed evidence. Preserve
v1 Library version history when replacing the final ZIP.

## Evidence and retrospective

Verification on Linux with Rust 1.99.1 and Python 3.12.14:

- cargo xtask check ci: formatting, architecture, docs, Clippy, 18 Rust tests,
  rustdoc, and six replay scenarios passed.
- PYTHONDONTWRITEBYTECODE=1 /tmp/rust-starter-gepa-venv/bin/python -m unittest
  discover -s experiments/tests -v: all 13 tests passed, including the real
  pinned GEPA package with a scripted proposer and a separate holdout comparison.
  The temporary environment was installed from the committed requirements.lock;
  create target/gepa-venv following experiments/gepa/README.md to reproduce it.
- python3 -m experiments.evaluator.demo --output target/experiments/demo-final:
  no-op rejected, correct Rust edit accepted, protected edit rejected.
- GitHub Actions includes a separate experiment-contracts job. Remote Actions
  and macOS execution were not run in this session.

The delivered archive includes selected reports and logs under target/ and a
VERIFICATION.txt index. Reports retain original absolute paths; use their relative
suffix within the extracted archive. No live coding-agent or paid GEPA run was
performed, and no canonical skill was promoted. This establishes functioning
measurement infrastructure, not measured model gains.

Revision: completed the planned implementation and local verification. Future
work is product-specific task datasets, a configured coding-agent adapter, and
an explicitly budgeted experiment. Fresh holdouts are required after tuning.
