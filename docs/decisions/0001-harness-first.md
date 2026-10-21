# ADR 0001: A small product and an executable harness

Status: accepted. Owner: maintainers.

## Context

The supplied article, “Harness engineering: leveraging Codex in an agent-first
world” by Ryan Lopopolo, describes repository-local knowledge, enforceable
boundaries, runtime legibility, execution plans, and continuous maintenance.
This template applies those principles to Rust without assuming its internal
application architecture, autonomous merge policy, or reported productivity
results transfer to every project.

## Decision

Use a small Cargo workspace, a Rust xtask command surface, strict JSON policy,
Cargo metadata for crate boundaries, short agent maps, owned documentation,
process scenarios, bounded execution, and evidence reports. Keep application
frameworks, databases, async runtimes, model vendors, and deployment out of the
default. The sample runtime is replaceable. Keep product and harness dependencies
separate. Pin a tested toolchain and commit Cargo.lock.

Use named check profiles rather than automatic “changed-files only” selection:
implicit dependency selection can skip affected contracts. Add selection only
when its correctness and time benefit have been measured.

Use standard Cargo facilities for workspace inheritance and structured metadata:
[Cargo workspaces](https://doc.rust-lang.org/cargo/reference/workspaces.html) and
[cargo metadata](https://doc.rust-lang.org/cargo/commands/cargo-metadata.html).

## Consequences

The starter is immediately runnable and understandable, but each product still
needs its own acceptance criteria, runtime driver, security controls, and useful
observability. The harness is developer tooling, not a sandbox or an AI agent.
Architecture checks operate at crate granularity. Dependency scanners, semantic
reviewers, and agent benchmarks are extensions, not implied guarantees.

Review minimal merge gates according to the product's actual risk. We do not
copy the article's autonomous merging or permissive treatment of flakes as a
universal default. Gate weakening needs a deliberate repository-owner decision.
