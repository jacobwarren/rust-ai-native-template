# Focus gap discovery on the assigned work

Status: implementation complete. Owner: maintainers.
Baseline: the first completion-gate delivery, based on upstream 7f63e90.

## Purpose and boundaries

Keep task coordination outside the verification harness. Preserve gap transition
checks, scoped acceptance and release gates, while removing the unused external
snapshot/provider/freshness implementation. Ticket references remain optional and
opaque. Agents investigate gaps; a stop checkpoint requires review and accounting,
not an automated audit or a minimum number of findings.

## Implementation and decisions

- The repository gap register is the core authority. Legacy external mode fails
  with a migration diagnostic rather than silently using a stale local copy.
- Schema v2 tasks add four review answers: acceptance, verification, assumptions
  and coverage. The answers can be drafted during work, but blanks block handoff.
- Historical v1 records remain readable. New claims require v2 review.
- No-discovery and prompt-only tasks pass with current evidence. Material findings
  need durable records; an out-of-scope gap may remain open after task completion.
- Keep interrupted/blocked work stoppable with explicit remaining actions.

## Verification and recovery

Focused completion, gap discovery and transition tests pass (18 tests). Their
passing/failing cases include omitted review answers, missing durable records,
opaque references, no new gaps, stale evidence and open release blockers.
Run `cargo xtask check ci` on final source, followed by completion and handoff for
completion-gate against 7f63e90. Reports go in target/verification. Re-run after
source edits. Final artifact results are outside source to avoid invalidating
verification by writing its own report.

## Outcome and remaining work

No known implementation blocker remains for this bounded change. Configure the
opt-in hook and hosted review authority when adopting. No live ticket integration,
remote CI execution, autonomous audit or exhaustive discovery claim is made.
The original three product adoption gaps remain open. The prior snapshot design
is recorded in 0004-completion-gate.md as historical context, superseded here.
