# one-brief

Written with the write-ticket skill (.claude/skills/write-ticket/SKILL.md).

## Problem
The template still ships machinery its own evidence says does not pay.
`docs/EVALUATION.md`, "Pages on product tasks": twelve graded runs were all
accepted with or without pages; six pages cost 95 and 126 minutes for features
a no-pages run delivered in 3. The full tier and the review-and-rewrite loop
are now opt-in, but an adopter still inherits them:

- `xtask/src/router.rs` (400 lines) and `handover.rs` (333): the page loop,
  the verdict log, three review rounds, the best-version rule.
- A reviewer agent and skill, eleven exemplar pages (408 KB), and a 17 KB
  `docs/ARTIFACTS.md`, most of it about pages nobody writes by default.
- Three tiers, a `reader` field and schema 6 compatibility in
  `xtask/src/task_record.rs`, for records the tree no longer holds.

A sceptical adopter reads this first, and the only large numbers in the
evaluation belong to it.

## Scope
- Remove the full tier, the `reader` rule, the review loop and the verdict log
  from `xtask/`: `router.rs` and `handover.rs` go; a small module keeps
  `task-artifact next` for the one page left. `task-artifact verdict` goes.
- `xtask/src/task_record.rs`: tiers are `small` and `standard`. A `reader`
  value in an existing record is still accepted and ignored. A record below
  schema 7 can be read but cannot make a new claim.
- `xtask/src/artifacts.rs`: one page, `brief.html`: stylesheet, text floor and
  commit order before code. The edit guard, `style` and `screenshot` stay.
- Remove the reviewer agent and the review-artifact skill (both hosts), the
  exemplars other than the plan page the brief is modelled on, and the
  calibration method that depended on them.
- `docs/ARTIFACTS.md` shrinks to the brief; `AGENTS.md`, `README.md`,
  `docs/COMPLETION.md`, `docs/EVALUATION.md`, `ARCHITECTURE.md`, the PR
  template, the remaining skills and `harness/readiness.json` follow.
- The branch `showcase/full-ceremony` names the last commit that has all of
  it, for anyone who wants the eight pages and the review loop back.

Tier: standard.

## Acceptance as named tests
- `tier_is_small_or_standard`: given a record with `"tier": "full"`, then it
  is refused by name; a record with `"reader"` still parses.
- `a_record_below_schema_7_cannot_claim`: given a schema 6 record, then
  `coordination` reads it and `complete` asks for schema 7.
- `a_standard_task_owes_a_brief_before_code`: given a standard task, then
  completion needs `brief.html`, with the stylesheet and the text floor,
  committed before the first product-code commit, and nothing else.
- `next_walks_a_standard_task_to_its_brief_and_no_further`: given a standard
  task, then `next` hands over the filled-in brief prompt, then says to commit
  the brief, then to build and complete; it never names a review.
- `the_page_commands_are_style_check_guard_next_and_screenshot`: given
  `cargo xtask help task-artifact`, then exactly those five are listed, and
  `task-artifact verdict` is an unknown command with their usage.

## Validation
- `cargo xtask check ci`: 9 of 9.
- `cargo xtask complete --task one-brief --base harness/pages-by-risk --report
  target/verification/ci.json`: complete.
- `wc -l xtask/src/*.rs` and `du -sh examples` before and after, in the
  record's review answers.

## Out of scope
- Removing task records, receipts or the brief itself: the comparison against
  a stripped copy decides those (`docs/EVALUATION.md`).
- Archiving this stack's own records and their old `report.html` pages.
- The `service-acceptance` gap.
