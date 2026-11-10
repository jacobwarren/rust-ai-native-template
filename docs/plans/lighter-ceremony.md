# lighter-ceremony

Written with the write-ticket skill (.claude/skills/write-ticket/SKILL.md).

## Problem
A review of the template found the task-page ceremony too heavy for most work,
weaker than it claims, and hard to enter:

- Five merged tasks wrote 3,235 to 5,159 lines of HTML for 209 to 773 changed
  lines elsewhere, with 12 to 17 logged page reviews each
  (`git diff --shortstat` per merge; `harness/task-artifacts/*/reviews.json`).
  Anything over 100 lines, or touching `xtask/`, a hook or a `Cargo.toml`,
  owes all eight pages (`xtask/src/size.rs`).
- The interview and the quiz are written for a person, and no person answered:
  `harness/tasks/agent-discoverability.json` records "The owner answered no
  discovery page".
- The open gap `artifact-gate-holes` (`harness/gaps.json`): PR CI checks page
  order from the merge commit, where `history::commits` reads no commits; order
  is checked only for tasks marked complete; a page holding only the stylesheet
  marker passes.
- `AGENTS.md` step 2 packs tiers, pages, three commands and two search tools
  into one item; a fresh agent opened 30 files to learn the harness
  (`docs/plans/agent-discoverability.md`).

## Scope
- `xtask/src/task_record.rs`: schema version 7; tiers `small`, `standard`
  (two pages, the default) and `full` (the eight-page set); `reader`
  (`human` or `none`). Schema 6 records keep their meaning.
- `xtask/src/artifacts.rs`, `router.rs`, `handover.rs`, `completion.rs`,
  `history.rs`, `main.rs`, `help.rs`: the page set per tier and reader; a text
  floor on required pages; before-code order at partial handoff;
  `handoff --head REV` and merge-commit detection.
- `docs/ARTIFACTS.md`: the three tiers, both standard prompts, the reader rule.
- `.github/workflows/ci.yml`: pass the PR head to handoff.
- `AGENTS.md`, `README.md`, `docs/COMPLETION.md`, the ceremony skills, the
  agent definitions, the examples and the PR template: say the same thing.
- `harness/gaps.json`: close `artifact-gate-holes` with its receipt.

Tier: standard, reader none. This task is the first one run under the tier it
adds: the branch's own gate judges it.

## Acceptance as named tests
- `standard_task_owes_a_brief_before_code_and_a_report_after`: given a schema 7
  record with no tier, then completion asks for `brief.html` before the first
  code commit and `report.html` at or after the last, and for no other page.
- `full_task_writes_the_interview_and_quiz_only_for_a_human_reader`: given a
  full task, then six pages complete it without a reader and eight with one,
  and the plan's hand-over says where the interview's questions go.
- `reader_and_tier_are_strict`: unknown tiers, readers and schema versions fail;
  a schema 6 record cannot name a reader.
- `a_placeholder_page_is_refused`: given a required page with under 1,200
  characters outside its tags and styles, then the check names the
  page and its count.
- `a_partial_handoff_checks_that_the_brief_came_before_code`: given a partial
  record whose brief was committed after code, then handoff refuses it.
- `handoff_reads_the_branchs_commits_from_a_pull_request_merge`: given a merge
  commit on the target branch, then handoff finds the late brief with `--head`,
  and without it when the merge sits on the base.
- `next_walks_a_standard_task_through_two_pages`: given a standard task, then
  `next` hands over the brief prompt with the no-reader and length rules, then
  the report prompt, and `verdict` refuses a page the task does not owe.

## Validation
- `cargo xtask check ci > target/verification/ci.json`: 9 of 9.
- `cargo xtask gaps --base origin/main`: ok with this branch's receipt.
- `cargo xtask complete --task lighter-ceremony --base origin/main --report
  target/verification/ci.json`: complete, judged by this branch's gate.

## Out of scope
- Measuring whether pages help: the paired run is its own task
  (`docs/EVALUATION.md`).
- Product work through the harness, and removing the template's own task
  history from the default branch: later tasks on stacked branches.
- A gate that forces the full tier. The tier is the owner's call; receipts
  already cover policy paths.
