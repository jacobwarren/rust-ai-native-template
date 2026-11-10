# pages-by-risk

Written with the write-ticket skill (.claude/skills/write-ticket/SKILL.md).

## Problem
A paired run measured what task pages buy on product work
(`docs/EVALUATION.md`, "Pages on product tasks"). Five board features were each
built by the same model with no pages and with the standard tier's brief and
report. All ten passed held-out checks and CI. The no-pages runs took about 3
minutes each; the standard runs 33 to 54. Most of the added time was the
review-and-rewrite loop, and most rewrites were for length or a citation, not
for the code. The report page took more review rounds than the brief in three
of five runs, and a late code fix forces it to be rewritten.

Where a page did find something, it was a brief on a change to the gates:
`harness/task-artifacts/lighter-ceremony/brief.html` (a crash and a floor that
refused six pages) and `harness/task-artifacts/ship-clean/brief.html` (an
archive check that would fail for anyone starting without history).

So the default still charges every task over 100 lines for two reviewed pages
that the evidence says help only on risky changes.

## Scope
- `xtask/src/task_record.rs`, `artifacts.rs`, `size.rs`, `router.rs`,
  `handover.rs`: the default tier is `small`, which owes no pages at any size
  and is refused only for policy, dependency and ceremony paths; `standard`
  owes one page, `brief.html`, before code. `report.html` is no longer a page.
- `docs/ARTIFACTS.md`, `AGENTS.md`, `README.md`, `docs/COMPLETION.md`, the
  ceremony skills: which changes owe a brief, that review of a brief is the
  owner's call, and that the record's four review answers are the report.
- `docs/EVALUATION.md`: the paired run, its limits, and the held-out build.
- `harness/gaps.json`: a gap for what the held-out build found, that the
  workflow runner cannot grade a long-running service.
- `scripts/agent-hook.sh`: the receipt hint names the base it cannot assume.

Tier: standard, reader none.

## Acceptance as named tests
- `a_task_with_no_tier_owes_no_pages_at_any_size`: given a schema 7 record
  with no tier and a 500-line product change, then completion asks for no page.
- `a_pageless_task_is_refused_on_policy_paths_only`: given a small task, then
  a change to `xtask/`, a `Cargo.toml` or a ceremony file is refused and tells
  the agent to restart as standard with a brief; 500 changed lines are not.
  Changed after the brief: the product's acceptance files (workflows,
  scenarios, readiness, goldens, `docs/PRODUCT.md`) are not refused either,
  since every feature edits them; they keep their receipt
  (`protected_cargo_and_ceremony_paths_force_standard`).
- `a_standard_task_owes_only_a_brief_before_code`: given a standard task, then
  completion needs `brief.html` before the first code commit and nothing after
  it; `report.html` is not a page a task owes.
- `next_offers_review_of_a_brief_without_requiring_it`: given an authored
  brief, then `next` names review as optional and the commit as the next step.

## Validation
- `cargo xtask check ci`: 9 of 9.
- `cargo xtask complete --task pages-by-risk --base harness/ship-clean
  --report target/verification/ci.json`: complete, judged by this branch's gate
  (a brief, no report).

## Out of scope
- Removing the full tier: it stays for a person who wants to steer a design.
- Teaching the workflow runner to grade a service: recorded as a gap.
- Rerunning the experiment on harder tasks, where a brief might help product
  work; `docs/EVALUATION.md` says what such a set needs.
