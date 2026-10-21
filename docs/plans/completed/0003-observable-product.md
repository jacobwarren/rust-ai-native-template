# Observable product and evidence-backed readiness

Status: completed. Owner: maintainers. Starting source: v2 b88ffeae3ced3cb3cf581e7d8666a417f32c89d255d68fa9f773172236889043.

## Purpose and acceptance

Give agents a real stateful Rust product to discover, operate, inspect, capture,
and replay. Keep the greeting compatible. Prove complete process workflows,
strict protocol admission, failure atomicity, persistence and interrupted-save
recovery. Gate readiness on actual evidence rather than feature labels.

## Context

crates/domain is deterministic; crates/app is the process boundary. Add
crates/engine for the typed command/session and storage boundary. xtask owns
independent workflow expectations, golden comparisons and release gating.
Normal development stays Rust-only; optional Python/GEPA remains separate.

## Progress

- [x] Read scoped instructions and establish doctor/fast baseline.
- [x] Implement board domain, typed session, discoverable JSON-lines driver.
- [x] Implement atomic persistence and test injected failures/interruption.
- [x] Add process workflows, reviewed baselines, property/differential tests.
- [x] Add repeatable regression procedure, ranked gaps, readiness gates and CI.
- [x] Run all checks, complete this plan, deliver updated archive with evidence.

## Decisions

Use a small task board (add, complete, remove, undo) rather than product-specific
GUI scaffolding. CLI and JSON-lines driver share one session API. Store a bounded,
versioned command journal; replay rebuilds state and undo. Save uses same-directory
atomic replacement and file/directory sync on supported Unix platforms. A root
lock prevents two live writers. Failed pre-commit saves preserve the previous
file; post-replace durability errors explicitly report uncertainty.

Golden comparisons never create missing expectations. Candidate updates are
separate artifacts. Release checks consume fresh source-bound CI evidence and
named workflow evidence; no deployment occurs. Product release readiness remains
blocked until real product requirements and platform acceptance are provided.

## Discoveries and evidence

Strict request tests found that Serde unit variants accepted unknown fields even
with deny_unknown_fields. Empty struct variants now reject those inputs. The
fail-before/fix-after record is docs/regressions/0001-strict-unit-commands.md;
the archive includes the original failing output as regression-before.log.

The full CI profile passed all eight stages. Four process workflows cover
lifecycle/restart/undo, rejected edits, atomic replay, and script/stdin parity.
Generated sequences compare 64 fixed seeds of 100 edits against an independent
model and replay each intermediate journal. Storage tests inject failures before
and after replacement and terminate a real child before commit. Negative grader
tests reject successful processes with wrong or missing golden state. Readiness
tests reject stale, incomplete, missing-step and missing-workflow evidence.
The existing 13 optional Python experiment contract tests passed without model
calls. GEPA remains optional and outside the production dependency graph.

Final commands: cargo xtask check ci, then cargo xtask release template --report
<ci-report>. The archive's VERIFICATION.txt identifies the final source digest,
counts and report locations; reports live under target/verification and
target/harness/runs. A product release check is expected to fail because real
product acceptance, platform qualification and hosting enforcement remain open.

## Outcome and limitations

The task board supplies a replaceable reference implementation, observable
protocol, persistence, recovery and independent evidence. Template readiness is
supported by local Linux checks. No remote CI or macOS execution, power-loss
qualification, hosting ruleset setup or paid model optimization is claimed.
Repository hosting settings require the adopter's actual remote and owners;
docs/ADOPTING.md records the exact setup. Plans, architecture, regression records
and ranked gaps provide durable entry points for subsequent agent work.
