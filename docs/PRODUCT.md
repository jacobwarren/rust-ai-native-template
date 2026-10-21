# Product intent and acceptance contract

Status: active example. Owner: maintainers. Replace this document during adoption.

The bundled product demonstrates one complete, observable Rust execution path.
It greets a validated name and provides a bounded task board with undo, replay
and persistence. Both are disposable examples; adopt real product requirements.

## Public behavior

- `starter-app greet <name>` accepts one argument, trims surrounding whitespace,
  rejects control characters before trimming, and requires 1–80 Unicode scalar
  values. Grapheme clusters are not counted; this choice is explicit.
- Success: exit 0, one stdout JSON object with a `message` field.
- Invalid input or invocation: exit 2, one stdout JSON object with an `error`
  containing stable `code` and human-readable `message` fields.
- Stderr: newline-delimited JSON event objects, including `event` and `run_id`.
  Logs exclude the raw name. They are deliberately minimal telemetry, not a
  distributed tracing implementation.
- The greeting has no storage/network effects. Domain behavior is deterministic.

Examples are executable in [scenarios](../harness/scenarios.json). Domain edge
conditions live in [unit tests](../crates/domain/src/lib.rs). Together these
provide a sample vertical slice to replace with real product behavior.

## Before building your product

Specify the user, observable outcomes, non-goals, supported platforms, important
failure modes, resource budgets, and authorization boundaries. Attach each
acceptance criterion to a test, replay, inspection, or explicitly manual check.
Do not call unfinished product requirements “implemented” because this template
passes its example checks.

## Stateful reference

The task board supports add, complete, remove and undo through one engine path.
Its driver discovers capabilities, inspects state, captures evidence, replays a
bounded journal and saves/reopens it. [DRIVER.md](DRIVER.md) defines protocol,
resource limits and persistence failure semantics. Complete process journeys live
in harness/workflows.json; baselines are independently authored under
harness/goldens. [TESTING.md](TESTING.md) records which boundary each check covers.

A usable example must preserve rejected edits, reopen the last committed state
after interrupted saving, preserve undo through replay/reopen, and produce the
same results through stdin and file-script adapters. It must never bless its own
missing baseline. Live app sessions are single-writer per storage root.

Release scope is defined in [READINESS.md](READINESS.md). Passing the template's
gate leaves the adopted-product blockers open until real evidence closes them.
