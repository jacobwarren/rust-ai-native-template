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
harness/docs.json. Tier: standard. Skills are ceremony paths.

## Acceptance as named tests
- `reviewer_brief_runs_the_fixed_checks`: given the review-artifact skill, then
  it lists the fixed checks every review runs (reply matches the page, output
  shown as captured was captured, every ticket requirement met or its change
  explained, an uncited claim fails even when the reviewer can confirm it), in
  both host copies.
- `author_brief_answers_each_fixed_check`: given the author-artifact skill,
  then each fixed check the reviewer runs has a matching instruction, so an
  author can meet it before review.

## Validation
`cargo xtask check ci > target/verification/ci.json`;
`cargo xtask complete --task brief-calibration --base origin/main --report target/verification/ci.json`;
rerun the calibration with the new reviewer brief on the saved next-prompt
drafts and committed pages (target/artifact-reviews/next-prompt/) and compare
with the table above: the subtle findings should be raised more often, and the
committed pages should still pass.

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

## Out of scope
A rewrite-rate command; changing the prompts, the exemplars or the gate;
running several reviewers per round; moving the reviewer to another model
(Opus at low effort gave the same verdicts at a higher price).
