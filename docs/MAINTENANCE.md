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
