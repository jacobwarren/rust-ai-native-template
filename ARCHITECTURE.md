# Architecture

This repository makes Rust changes navigable, executable, and independently
checkable. It contains disposable greeting and stateful task-board examples, a Rust verification
harness, and optional agent-evaluation experiments. The normal build has no model
or Python dependency. Product intent lives in docs/PRODUCT.md.

## Code map: where to make a change

| Concern | Location and important symbols | Responsibility |
| --- | --- | --- |
| Input invariants and business behavior | crates/domain/src/lib.rs: Name, NameError, greet | Parse once; deterministic domain behavior |
| Stateful domain | crates/domain/src/board.rs: Title, Board | Validated deterministic edits; no serialization or I/O |
| Shared application path | crates/engine/src/lib.rs: Command, Session | Prepare/commit edits, undo history, atomic replay |
| Application driver | crates/engine/src/protocol.rs: Driver, Request | Discover, execute, inspect, capture, replay, save/load |
| Persistence | crates/engine/src/store.rs: Store | Writer lock, bounded versioned journal, atomic replacement |
| CLI protocol and telemetry | crates/app/src/main.rs and driver.rs: main, serve | Arguments, bounded JSON-lines/script transport, exit codes/events |
| Command navigation | xtask/src/main.rs: dispatch | Public harness command interface |
| Policy and command admission | xtask/src/config.rs: Config, RawStep, Step | Strict wire parsing into valid private command specifications |
| Bounded execution and reports | xtask/src/runner.rs: Status, Outcome, execute, profile | Process lifecycle, logs, checkpoint/final reports |
| Architecture and knowledge checks | xtask/src/checks.rs: architecture_errors, docs | Dependency policy and document structure |
| Application acceptance | xtask/src/scenarios.rs: run | Exact CLI output, exit, and event expectations |
| Workflow acceptance and goldens | xtask/src/workflows.rs, golden.rs | Independent process expectations and read-only baseline comparison |
| Readiness and work selection | xtask/src/readiness.rs: assess, list_gaps | Current evidence, named scope requirements and ranked blockers |
| Source identity | xtask/src/repo.rs: metadata, fingerprint, worktree_id | Workspace discovery and inspected-source identity |
| Independent agent trials | experiments/evaluator: Suite, Task, evaluate | Frozen task, disposable checkout, permitted patch, separate grading tree |
| Skill search and final evaluation | experiments/gepa: Experiment, optimize, compare | Optional GEPA adapter and explicit paired holdout comparison |

Search these symbols when navigating. Details belong beside the implementation
or in operational docs; this map records stable responsibilities.

## Dependency boundaries

| Crate | Layer | Allowed internal dependencies | External dependencies |
| --- | --- | --- | --- |
| starter-domain | domain | None | None |
| starter-engine | engine | domain | serde, serde_json, sha2 |
| starter-app | runtime | domain, engine | serde_json |
| xtask | tooling | None | serde, serde_json, sha2 |

The source of truth for allowed edges is [harness/config.json](harness/config.json).
`cargo xtask architecture` reads Cargo's resolved workspace metadata and checks
all declared direct dependencies, including renamed, dev, build, optional, and
target-specific dependencies. Each member must declare a known layer and owner.
Local path dependencies outside the governed workspace are rejected.

Edges describe “may depend on”, not execution order. Transitive external
packages are governed by Cargo.lock and dependency review, not this allowlist.
Cargo catches dependency cycles; this policy does not replace the compiler.

## Boundary rules

1. Domain types validate invariants; external strings do not circulate unchecked.
2. Engine owns commands/session and storage; runtime owns transport and telemetry.
   Every adapter invokes Driver/Session. The harness grades the public process
   without depending on the product engine crate.
3. Tooling never becomes a product dependency.
4. Keep Rust files below the configured 400-line ceiling. This is a navigation
   budget, not a metric to optimize by compressing code or splitting arbitrarily.
5. Shared workspace lints forbid unsafe code by default. CI also passes the
   critical rules explicitly to Clippy across all workspace targets.

Crate dependency checks do not prove that domain code avoids standard-library
I/O, that a module has no side effects, or that secrets cannot flow. Add AST or
behavioral checks when a real failure justifies them. Do not pretend substring
searches establish those properties.

## Grow only when needed

- **Library:** make the domain or new library the public target; replace CLI
  scenarios with external-consumer integration tests and documented examples.
- **Service:** add transport and infrastructure crates, readiness probes,
  graceful shutdown, port-zero startup, request replay, and local state isolation.
- **CLI:** replace the example's arguments and output contract; use typed parsing
  when complexity warrants a parser dependency.
- **Desktop/game/embedded:** add a platform runtime and platform-specific
  verification profiles. Keep headless domain checks fast; device and graphical
  checks require their own runtime driver and artifacts.
- **Many domains:** separate stable business boundaries, then add their exact
  allowed edges. Avoid a universal utils crate or a trait for every function.

Add or rename a crate in Cargo.toml, its metadata, the harness policy, the map,
and affected profiles/scenarios together. Record consequential changes in an
[ADR](docs/decisions/0001-harness-first.md).

## Cross-cutting concerns

Configuration enters through strict parsing. Step has private fields, a separate
executable and argument list, and NonZeroU64 timeout; execution cannot receive an
empty argv through its public API. Status is an enum serialized to stable JSON
strings. Domain errors are typed; harness infrastructure errors retain context
in diagnostic strings and failed reports.

The runtime owns telemetry. The harness supplies run IDs and isolated local
state paths; products must honor them. Repository inspection and fingerprints
identify evidence, but do not attest to hostile-process behavior.

The evaluator treats tasks, regression commands, and grader bytes as controls.
It copies only explicitly allowed source changes into a fresh grading tree.
The optimizer can propose one skill and receives bounded diagnostic feedback;
it cannot redefine the evaluator's scoring contract through that interface.
Same-user local processes are still not a security sandbox. Holdout source and
credentials require separate OS/service isolation against adversarial agents.

Python is confined to experiments/. Neither product crates nor xtask depend on
GEPA. Optional experiment tests run separately in CI. No candidate is promoted
automatically. Detailed protocols live in experiments/README.md and
experiments/gepa/README.md.

The stateful example commits an edit or replay only after successful validation.
Storage has a distinct commit point: errors before rename preserve the previous
file; errors after rename report uncertain durability. Bounded journals make
undo and replay reproducible. Detailed contracts live in docs/DRIVER.md.

Application changes carry workflow evidence at the boundary they actually reach.
Golden comparison is read-only; missing baselines fail. Readiness consumes a
current CI report and named test/workflow outcomes. Docs, plans and essential
operating knowledge remain checked in; no private plan directory is required.
