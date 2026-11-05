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
`xtask/src/handover.rs` (new), `xtask/src/main.rs` (`--notes -`),
`xtask/src/task_record.rs` (the optional `prompt` object), the write-ticket
skill in both hosts, `xtask/tests/handover.rs` (new), `docs/ARTIFACTS.md` (one
source for the prompt text), README, `docs/COMPLETION.md`,
`examples/task-record.json`, `harness/readiness.json`, `harness/docs.json`.
Tier: standard. It touches protected xtask paths and docs/ARTIFACTS.md, a
ceremony path.

## Acceptance as named tests
Revised by the plan page: the values come from the record alone, and a rewrite
gets the last review's full notes instead of the kept version's path.
- `next_prompts_match_the_artifacts_guide`: the guide's File column lists the
  eight pages in order, every brace is one `next` fills, and only the blindspot
  prompt opens "I'm about to {task}".
- `prompt_values_are_optional_and_strict`: a missing, null or blank `prompt`
  value parses; an unknown key inside `prompt` is refused.
- `next_hands_over_the_filled_in_prompt`: with every value set, the author step
  carries the filled prompt, exemplar, output path, task context and pages so
  far, with nothing unfilled.
- `build_step_hands_over_the_notes_prompt`: after the discovery pages, the build
  step carries the implementation-notes hand-over.
- `missing_values_stay_as_braces`: with no value, or a `<…>` placeholder, the
  brace stays in the text and is listed in `unfilled`; the exit code is 0.
- `verdict_reads_full_notes_from_stdin`: `--notes -` takes up to 64 KiB from
  stdin; more, empty or non-UTF-8 input is refused.
- `review_and_rewrite_steps_carry_their_prompt`: the review step carries the
  prompt and ends its say with `--notes -`; after a revise piped through stdin,
  the rewrite carries those notes as written.

## Validation
`cargo xtask check ci > target/verification/ci.json`;
`cargo xtask complete --task next-prompt --base origin/main --report target/verification/ci.json`;
this task's own pages after the first product slice are started from the prompt
`next` prints.

## Out of scope
Computing rewrite rates from reviews.json (a later ticket); making completion
read the log; changing the prompts' wording or the exemplars; starting
subagents from xtask.
