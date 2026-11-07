# brief-calibration

Written with the write-ticket skill (.claude/skills/write-ticket/SKILL.md).

## Problem
The page reviewer has never been calibrated, though docs/EVALUATION.md asks
that of any semantic checker. A first calibration (21 valid fresh reviews of
next-prompt's reference, plan and buy-in pages: three saved round-1 drafts and
the three committed pages) found the verdicts stable: 19 of 21 matched the
original pass or revise. The findings were not. Every run caught a page that
ran long or re-listed earlier pages. Problems where a page disagrees with its
own evidence were caught one to two times in three:

| Original finding | Fresh runs that raised it |
| --- | --- |
| reference draft: the copy-back reply says Q5 stands while the page's table refines it | 2 of 3 |
| buy-in draft: main-side demo output reconstructed but shown as captured | 2 of 3 |
| buy-in draft: "9/9 CI checks pass" tile cites no run | 1 of 3 (two ran `check ci` themselves and accepted it) |
| plan page: a ticket requirement (the kept path) dropped with no reason | 1 of 4 |

The author brief (.claude/skills/author-artifact/SKILL.md) guards against none
of these, and the reviewer brief leaves them to whichever claims a reviewer
happens to sample. Across the three tasks logged so far, reference, plan and
buy-in needed a rewrite every time, mostly for length.

## Scope
The author and reviewer briefs in both hosts (.claude/skills and .agents/skills:
author-artifact, review-artifact), docs/EVALUATION.md (the calibration method
and when to rerun it), xtask/tests/briefs.rs (new), harness/readiness.json,
harness/docs.json, and (added by the plan page) the three round-1 drafts copied
to harness/calibration/next-prompt/ so a clone can rerun the calibration.
Tier: standard. Skills are ceremony paths.

## Acceptance as named tests
Revised by the plan page: the reply and captured-output checks are widened to
`consistent` and `provenance` (two places on a page disagree; something is
shown as more real than it is), which also cover three logged failures the
calibration never sampled.
- `reviewer_brief_runs_the_fixed_checks`: given the review-artifact skill, then
  it lists the fixed checks every review runs (`consistent`, `provenance`,
  `cited`: a number or claim with no source or command on the page fails even
  when the reviewer can confirm it, and `ticket`: every ticket item met or its
  change explained), in both host copies.
- `author_brief_answers_each_fixed_check`: given the author-artifact skill,
  then each fixed check the reviewer runs has a matching instruction, so an
  author can meet it before review.

## Validation
`cargo xtask check ci > target/verification/ci.json`;
`cargo xtask complete --task brief-calibration --base origin/main --report target/verification/ci.json`;
rerun the calibration with the new reviewer brief, three runs per page, against
the expected verdicts in docs/EVALUATION.md (revised by the plan page: the
committed plan page is now expected to fail the `ticket` check, so "committed
pages should still pass" holds for reference and buy-in only).

## Calibration record (2026-11-06)
Each run was a fresh artifact-reviewer on Sonnet, with the unchanged
review-artifact skill and the page's own prompt, judging claims against the
commit the page was reviewed at. Nothing was recorded in a verdict log. The
round-1 drafts were the saved copies in target/artifact-reviews/next-prompt/.
"Opus" is the same agent on Opus at low effort.

| Page | Original | Fresh runs |
| --- | --- | --- |
| reference draft (8f1a6e6) | revise: reply says Q5 stands; edge row about the page's own prompt; stitched excerpt | revise (reply, length, strawman choices); pass; revise (reply, own-prompt row) |
| plan draft (cdaeb0f) | revise: 5,914 px against 3,678; process talk | revise ×3 (length and carried list, process talk) |
| buy-in draft (1002bb0) | revise: uncited 9/9 CI; reconstructed main output; length; open calls | revise (all four); revise (length; ran `check ci`, accepted the tile); revise (reconstructed output; length; ran `check ci`, accepted the tile) |
| reference page (cdaeb0f) | pass | pass ×3; Opus pass |
| plan page (3c1aa8e) | pass | pass; pass with 2 minor; revise (kept path dropped, length, prose data model); Opus pass |
| buy-in page (17425b6) | pass | pass ×3; Opus pass |

A first batch of four reference reviews was handed the record's later
one-reference prompt instead of the four-reference prompt the page answers.
All four failed it on that ground, and on length. They are left out of the
counts above. The result shows how strongly the given prompt steers a verdict.

## Rerun with the fixed checks (2026-11-06)
Round 1 ran 528a964 on the three drafts. Round 2 ran d4be473 (step 2 reworded)
on the three committed pages, three runs each, plus one more run per draft.
Expected verdicts are the ones in docs/EVALUATION.md, written before either
round.

| Page | Expected | Round 1 | Round 2 | Targeted finding raised |
| --- | --- | --- | --- | --- |
| reference draft | revise | revise ×3 | revise | contradicting reply 4 of 4 (was 2 of 3) |
| plan draft | revise | revise ×3 | revise | length 4 of 4 (was 3 of 3) |
| buy-in draft | revise | revise ×3 | revise | typed main output 4 of 4 (was 2 of 3); uncited 9 / 9 tile 4 of 4 (was 1 of 3) |
| plan page | revise | | revise ×3 | dropped kept path 3 of 3 (was 1 of 4) |
| reference page | pass | | pass, revise ×2 | |
| buy-in page | pass | | revise ×3 | |

Every defect the fixed checks target was raised every time. The cost is on
pages expected to pass. Their failures, as checked against the files:

- Real, missed by all earlier reviews. The buy-in page's "With the leans as
  they stand, nothing changes before merge" (buy-in.html:306) is fixed text
  the choices never update (`consistent`, 2 of 3). Its "So far one real reply
  has gone through" (:291) contradicts the review log at 17425b6, which holds
  two piped replies for implementation.html (`consistent`, 2 of 3).
- Real, by the stricter rule. The reference page shows `$`-prompted output
  from scratch probes (`probe_guide`, a patched `Task`) that exist nowhere a
  reader can run (`provenance` and `cited`, 2 of 3).
- Stricter than written. Two of three buy-in runs failed its tiles as
  uncited although `cargo xtask check ci` is named in the sentence beneath
  them, which the plan counts as a source. The `ticket` check also fired on
  the buy-in page for ticket changes it reports only as decisions. These are
  known gaps, not tuned away.

Round 3 ran d4be473 again on each draft, two more runs each, handed "record no
verdict in any log" instead of "do not run any task-artifact command". All six
revised, all six rendered, and each raised its draft's targeted findings: the
contradicting reply on the reference draft 2 of 2, length on the plan draft
2 of 2, and the typed main output and the uncited tiles on the buy-in draft
2 of 2. With round 2 that makes 3 of 3 per draft at the final wording.

Rendering: in round 1, six of nine runs skipped the screenshot step, reading
"before you run anything" as "run nothing". With d4be473, ten of twelve
rendered. One of the two that did not cited this calibration's own "do not
run any task-artifact command" instruction, which real reviews do not get.

## Out of scope
A rewrite-rate command; changing the prompts, the exemplars or the gate;
running several reviewers per round; moving the reviewer to another model
(Opus at low effort gave the same verdicts at a higher price).
