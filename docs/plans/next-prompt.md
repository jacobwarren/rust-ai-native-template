# next-prompt

Written with the write-ticket skill (.claude/skills/write-ticket/SKILL.md).

## Problem
`cargo xtask task-artifact next` names the step, the page, the round and which
subagent to start, but not what to tell it. Its author step says "with its prompt
and exemplar (docs/ARTIFACTS.md) and the task context" (`xtask/src/router.rs`,
`page_step`). So for every one of the eight pages, and for each review and
rewrite, the orchestrating agent looks up the page's prompt in the ARTIFACTS.md
table, fills its braces ({task}, {area}, {problem}, {reference}), finds the
exemplar, and assembles the reviewer's findings for a rewrite by hand. That
lookup is where the loop drifted before workflow-router, and it is the part a
cheaper model would get wrong.

## Scope
`xtask/src/router.rs` (the step JSON gains the prompt to hand over),
`xtask/tests/router.rs`, possibly `xtask/src/task_record.rs` or the ticket
skill if the brace values need a home, `docs/ARTIFACTS.md` (one source for the
prompt text), README, `harness/readiness.json`, `harness/docs.json`.
Tier: standard. It touches protected xtask paths and docs/ARTIFACTS.md, a
ceremony path.

## Acceptance as named tests
- `next_hands_over_the_filled_in_prompt`: given a task whose record and ticket
  supply the prompt inputs, when `next` says author for a page, then its step
  carries that page's prompt with every brace filled, the exemplar path and the
  output path, and no brace is left unfilled.
- `review_and_rewrite_steps_carry_their_prompt`: given a page under review or
  after a revise, then the step carries the page's prompt for the reviewer, and a
  rewrite step carries the last review's notes and the kept version's path.
- `next_prompts_match_the_artifacts_guide`: the prompt text `next` hands over is
  the text in docs/ARTIFACTS.md's table, so the two cannot drift.

## Validation
`cargo xtask check ci > target/verification/ci.json`;
`cargo xtask complete --task next-prompt --base origin/main --report target/verification/ci.json`;
this task's own pages after the first product slice are started from the prompt
`next` prints.

## Out of scope
Computing rewrite rates from reviews.json (a later ticket); making completion
read the log; changing the prompts' wording or the exemplars; starting
subagents from xtask.
