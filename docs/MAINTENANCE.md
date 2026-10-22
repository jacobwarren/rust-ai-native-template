# Semantic maintenance procedure

Owner: maintainers. Trigger: after a meaningful product change, a recurring agent
failure, or the scheduled structural check. The workflow supplies evidence; a
maintainer or explicitly assigned agent performs this review. No paid model is
scheduled by this template.

## Review one bounded area

Choose an active product contract, an architecture boundary, or a repeated failure.
Read its current docs and the responsible implementation and tests. Compare the
actual behavior with each material documented claim. Use the architecture map to
find the owner and the concrete entry points. Run the smallest command or scenario
that can resolve uncertainty; record unknowns rather than guessing.

Check whether names and responsibilities still match the architecture map, whether
plans accurately describe completed and unfinished work, and whether the quality
inventory claims checks that are actually executed. Inspect repeated suppressions,
workarounds, flaky tests, and agent retry loops for a missing capability.

## Produce reviewable findings

For each finding record the claim, source evidence, consequence, and proposed
correction. Distinguish stale documentation, a product defect, and an inadequate
check. Repair a small confirmed issue in the same change when authorized. For a
new guardrail, demonstrate both a real violation and a benign case it should allow.
Do not rewrite broad areas for stylistic uniformity or manufacture cleanup work.

## Close the loop

Update the affected contract, decision, or quality row with the actual verification
scope. If unresolved, put the obligation and closure criterion in the debt tracker.
Keep a completed plan for substantive investigations. Measure recurring failures,
false alarms, diagnosis time, and maintenance effort before increasing complexity.
A scheduled green build establishes structural health, not semantic freshness.

Use cargo xtask gaps to select the highest-impact applicable unresolved problem.
Retain minimized reproducers using [REGRESSIONS.md](REGRESSIONS.md). Gap closure
needs a checked-in resolution; recompute readiness with fresh full-CI evidence.

## Task and gap reconciliation

At task start, material discovery, milestone completion and handoff, reconcile
relevant gaps and durable discoveries. Structural scheduling does not discover
new blockers or establish semantic truth. Use [COMPLETION.md](COMPLETION.md) for
transition receipts and bounded task review; do not rewrite every
gap after each edit or re-verify historical tasks on every maintenance run.

## Bounded gap discovery review

Use a dedicated review task when broader discovery is wanted. Specify one area
(for example persistence recovery), the requirements and paths to inspect, a time
or scope budget, and the evidence to collect. The agent should:

1. Read the relevant contracts and existing gaps, then exercise representative
   success, rejection and recovery paths at the claimed runtime boundary.
2. Investigate failures and suspicious assumptions; distinguish observed defects,
   missing evidence and hypotheses. Preserve a reproducer where possible.
3. Update existing gaps or record only material unresolved findings with impact,
   evidence and next action. Report reviewed scope even when no findings result.
4. Reconcile the review task and hand off remaining investigation explicitly.

A useful assignment is: "Review persistence recovery against docs/DRIVER.md.
Exercise interrupted saves and reopening the last committed state. Record
reproducible shortcomings or missing evidence; do not expand into unrelated
features or invent gaps to fill a quota. Report what you inspected and what
remains unverified."

Do not run this review on every stop hook. A scheduled review requires explicit
scheduling and an agent runner; the existing scheduled workflow only runs
structural checks. No automated semantic auditor is installed by this template.
