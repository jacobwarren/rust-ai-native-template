# Plan: evidence-backed task handoff

Status: completed implementation. Owner: maintainers. Starting commit: 7f63e90.

## Purpose and non-goals

Add deterministic gap transition validation and task reconciliation before an
agent claims completion. Retain repository-managed gaps by default and support
external tracker snapshots without credentials or a duplicate manual backlog.
No tracker writes, universal hook interception, or claim of semantic completeness.

## Milestones and acceptance

1. Strict local/external gap loading; stale or incomplete snapshots block release.
2. Compare with Git merge base; bind consequential gap and policy changes to
   explicit rationale and evidence. Reject missing history and stale rationale.
3. Task records link acceptance, gaps, discoveries and documentation. Complete
   requires current CI evidence; partial/blocked handoffs remain possible.
4. Provider-neutral hook entry point and PR CI reconciliation. All source edits
   precede final verification. Documentation explains hosting and trust limits.

## Progress

- [x] Inspect upstream, policies, readiness verifier and CI.
- [x] Implement and test milestones; 15 focused admission tests pass.
- [ ] Re-run full Rust gate, optional Python contracts and task completion on final source.
- [ ] Package source, patch and reproducible verification evidence.

## Discoveries and decisions

Use source-owned task records for evidence links, not ticket descriptions. A
snapshot exporter is project-specific; core Rust validates its offline contract.
Completion reports live in target so writing reports cannot invalidate source.
Final task records must be written before CI to avoid circular fingerprints.

## Recovery and next action

Implementation is complete. Re-run `cargo xtask check ci`, then completion and
handoff against base 7f63e90. Reports belong under target/verification. Re-run
after any source edit. No upstream writes are required for ZIP delivery.

## Evidence and outcome

Focused regression tests pass for omissions, stale reports, weakened blockers,
external snapshot failures and honest partial handoffs. Full final-source
verification and packaging are recorded in target/verification so writing those
results does not invalidate the verified source. Local hooks are optional and
provider exports remain a trusted adoption integration; no live tracker or
remote CI result is claimed. The task record is harness/tasks/completion-gate.json.
