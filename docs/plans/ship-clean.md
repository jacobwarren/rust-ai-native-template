# ship-clean

Written with the write-ticket skill (.claude/skills/write-ticket/SKILL.md).

## Problem
Two things stand between this template and a new adopter.

The default branch carries the template's own history as if it were the
adopter's: eleven task records, ten receipts and 2.7 MB of task pages
(`du -sh harness/task-artifacts`), calibration drafts and nine tickets under
`docs/plans/`. The gates forbid removing them: handoff reports `task history
deleted` (`xtask/src/completion.rs`) and `gaps --base` fails with `transition
history changed or deleted` (`xtask/src/transitions.rs`).

Twelve agent runs of board features and two harness tasks under the new tiers
reported the same friction more than once:

- `cargo xtask check ci > target/verification/ci.json` fails in a fresh clone:
  the directory does not exist (five of seven run reports).
- A complete record without `"remaining": []` fails with `missing field
  remaining`, and the failing CI check does not say why (two reports).
- A failed check names only its log path; agents opened the log to find the
  failing test (three reports).
- "About half its exemplar's length" was measured two ways by consecutive
  reviewers of one page: round 1 read it as 62% of its exemplar by height, round
  2 failed it at 21,515 of 26,697 bytes after its author had cut it to 54% by
  height (`harness/task-artifacts/lighter-ceremony/reviews.json`).
- Handoff on a merge commit built after the target moved, without `--head`,
  refuses a correct branch and blames the brief
  (`harness/task-artifacts/lighter-ceremony/report.html`).
- `xtask/src/size.rs` and `scripts/agent-artifact-write.sh` still say
  "discovery pages"; the add-product-command skill names a branch that does
  not exist and does not warn that the discovery test runs every example
  against one seed board (three reports).

## Scope
- `xtask/src/archive.rs` (new), `completion.rs`, `transitions.rs`:
  `harness/archive.json` names a commit and the paths archived there; a task
  record, receipt or page may be deleted only when the manifest lists it and
  the commit holds it.
- Remove the eleven merged tasks' records, pages and receipts, the calibration
  drafts and their tickets and completed plans from the tree; the branch
  `showcase/template-history` names the commit that keeps them. Records from
  this stack stay.
- `xtask/src/runner.rs`: a profile run also writes its report to
  `target/verification/PROFILE.json`, and a failed check carries the tail of
  its output.
- `xtask/src/task_record.rs`: `remaining` defaults to empty.
- `docs/ARTIFACTS.md`, `xtask/src/router.rs`: the standard length rule becomes
  a file-size limit that `next` reports.
- `xtask/src/artifacts.rs`: the late-brief error mentions `--head` when HEAD is
  a merge commit.
- The stale messages and the add-product-command skill.
- `docs/EVALUATION.md`, `README.md`, `docs/ADOPTING.md`: where the history is.

Tier: standard, reader none.

## Acceptance as named tests
- `archived_history_may_be_deleted_and_nothing_else`: given a manifest that
  lists a task's record, pages and receipt at a commit holding them, then
  handoff and `gaps --base` accept their deletion; an unlisted record, a
  listed path the commit lacks, and an unknown commit are each refused.
- `a_profile_run_writes_its_report_where_the_docs_say`: given `check ci` in a
  tree with no `target/verification`, then `target/verification/ci.json`
  exists and equals the printed report.
- `a_failed_check_shows_the_end_of_its_output`: given a failing step, then its
  entry carries the last lines of its stdout and stderr.
- `a_complete_record_needs_no_remaining_field`: given a complete record with
  no `remaining`, then it validates; a partial one without it is still refused
  with a message that names the field.
- `next_reports_a_standard_page_over_its_size_limit`: given a standard page
  over the limit, then `next` exits 3 with an advisory naming the page, its
  size and the limit, and the hand-over states the limit in bytes.
- `a_late_brief_on_a_merge_commit_points_at_head`: given handoff without
  `--head` on a merge commit, then the order error says to pass `--head`.

## Validation
- `cargo xtask check ci`: 9 of 9, report at `target/verification/ci.json`.
- `cargo xtask handoff --base product/board-stats --report
  target/verification/ci.json`: ok with the deletions.
- `git ls-tree -r --name-only showcase/template-history -- harness/tasks`
  lists the eleven archived records.

## Out of scope
- Archiving this stack's own records; a later cleanup can, once they merge.
- Verifying that the archive branch is pushed: the commit is an ancestor of
  the default branch, so its objects travel with it.
- Changing the review loop. One-line factual corrections by hand are
  documented, not gated.
