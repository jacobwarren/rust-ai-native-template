# Rust Agent Starter

A Rust workspace built around a legible, executable engineering harness.
The greeting and task-board applications are deliberately small. The reusable asset is the feedback loop:
intent → implementation → bounded verification → evidence → review.

## Start

Prerequisites: Linux or macOS, Git, and Rust installed through rustup.
The pinned toolchain is **1.99.1** with rustfmt and Clippy. Cargo downloads the
locked dependencies on first use; after fetching, `CARGO_NET_OFFLINE=true` works.
Normal Cargo development requires no Python, Node, Docker, model SDK, or paid API.
Optional experiments use Python; GEPA is installed only when explicitly requested.

```sh
cd rust-agent-starter
git init
cargo xtask doctor
cargo xtask check ci
cargo xtask run -- greet Ada
```

The first Cargo invocation installs the pinned toolchain if needed.
`cargo xtask` always uses `--locked`; commit Cargo.lock. `cargo fmt --all`
formats code. Verification commands check formatting without changing files.
After customizing the project, commit the baseline and mark the hosted repo as a
GitHub template if desired. See [adoption steps](docs/ADOPTING.md).

## Included

| Surface | Working implementation |
| --- | --- |
| Agent context | Short AGENTS.md, scoped instructions, canonical docs, Claude pointer |
| Rust foundation | Edition 2024 workspace, pinned toolchain, lockfile, shared lints |
| Architecture | Cargo metadata dependency allowlists, owners, unregistered member failures |
| Knowledge checks | Markdown catalog, local inline links, AGENTS length |
| Runtime evidence | JSON output/logs, run IDs, isolated per-command state |
| Verification | Named profiles, timeouts, process groups, logs, source fingerprints |
| Behavior | Six CLI replay cases with exact exit/output and telemetry assertions |
| Harness regression | Negative architecture, docs, process, and CLI tests |
| Delivery | Same CI command locally and on Linux/macOS GitHub Actions |
| Stateful reference | Discoverable JSON driver, undo, replay, single-writer atomic persistence |
| Product evidence | Complete process workflows, explicit golden baselines, failure injection |
| Readiness | Source-bound test/workflow evidence, ranked gaps, named release scopes |
| Evolution | Plans, decision record, quality inventory, maintenance and evaluation protocol |

## Commands

| Command | Purpose |
| --- | --- |
| `cargo xtask doctor` | Check tool availability and configuration |
| `cargo xtask map` | Inspect packages, layers, owners, and declared dependencies |
| `cargo xtask check fast` | Formatting, architecture, docs, workspace tests |
| `cargo xtask check ci` | Fast checks plus Clippy, rustdoc, and replay scenarios |
| `cargo xtask architecture` | Inspect layer violations directly |
| `cargo xtask docs` | Inspect knowledge structure violations directly |
| `cargo xtask scenario` | Build and replay the application boundary |
| `cargo xtask run -- greet Ada` | Run example with isolated state and captured logs |
| `cargo xtask workflow [id]` | Run stateful journeys and retain evidence |
| `cargo xtask run -- capabilities` | Discover the application command surface |
| `cargo xtask run -- script examples/board.jsonl` | Run the stateful example with isolated persistence |
| `cargo xtask gaps` | Inspect ranked work and release blockers |
| `cargo xtask readiness --report <path>` | Inspect implemented/verified requirements and scopes |
| `cargo xtask release template --report <path>` | Fail unless fresh evidence qualifies the example/template |
| `cargo xtask env` | Inspect root, worktree ID, artifacts, and source fingerprint |

Commands emit JSON on stdout and use nonzero exit codes for failures. Cargo
bootstrap/compiler failures can occur before xtask starts; preserve their stderr.
Progress is on stderr. Each profile run writes a self-contained report and logs
under `target/harness/runs/<run-id>/`. Reports include log paths and command argv.
Do not commit runtime logs; attach selected, reviewed evidence to a PR.

## Layout

```text
AGENTS.md               short context map
ARCHITECTURE.md          boundaries and extension rules
crates/domain/          pure validated domain example
crates/engine/          shared commands, state, driver and persistence
crates/app/             replaceable CLI runtime
xtask/                  Rust harness implementation
harness/config.json     layers, dependencies, named command profiles
harness/scenarios.json  replayable public behavior contracts
harness/docs.json       document ownership and verification catalog
docs/                   intent, decisions, plans, quality, operating procedures
.github/workflows/      verification and scheduled structural maintenance
```

Start an agent with: “Read AGENTS.md. Implement [outcome]. Acceptance criteria:
[observable behavior]. Verify with the repository harness and report evidence.”
For a new product, use the [first-task prompt](docs/ADOPTING.md).

This is an executable starting point, not an assertion that every Rust product
needs these exact crates or that the harness improves every model. Read the
[known limits](docs/HARNESS.md) and [evaluation protocol](docs/EVALUATION.md).
The greeting is disposable; keep the feedback loop.

## Optional agent evaluation and optimization

The starter includes a separately invoked task evaluator and GEPA adapter.
Start with the [experiment guide](experiments/README.md). The self-contained demo
requires Python 3.11+ and Git, and tests real Rust behavior with scripted agents:

```sh
python3 -m experiments.evaluator.demo --output target/experiments/demo
```

GEPA installation, budgets, split controls, and the separate final comparison are
in [the GEPA guide](experiments/gepa/README.md). Normal CI makes no model calls.
Only measured, independently reviewed improvements should replace the canonical
[repository-investigation skill](docs/skills/investigate.md).

## Product feedback loop

Start with [the driver](docs/DRIVER.md), [behavioral testing](docs/TESTING.md),
[regression repair](docs/REGRESSIONS.md) and [readiness](docs/READINESS.md).
Capture the full gate's JSON output and preserve its referenced logs:

```sh
mkdir -p target/verification
cargo xtask check ci > target/verification/ci.json
cargo xtask release template --report target/verification/ci.json
```

The product scope intentionally stays blocked until real product requirements,
platform results and hosted enforcement are supplied. No paid model or deployment
is part of a release-readiness check. CI exposes a final required status for the
adopter to make mandatory in repository settings.
