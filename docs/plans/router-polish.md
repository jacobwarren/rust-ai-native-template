# router-polish

Written with the write-ticket skill (.claude/skills/write-ticket/SKILL.md).

## Problem
Three loose ends from workflow-router (PR #4):
1. While `reviews.json` has uncommitted rows, `cargo xtask task-artifact next` names the first passed page in table order as the page to commit, not the page just reviewed (buy-in review, harness/task-artifacts/workflow-router/reviews.json). It converges one step later.
2. Review log rows carry no finding count and no reviewer model, so rewrite rates per page type and per model cannot be computed from the log.
3. `docs/skills/investigate.md` is plain Markdown that agents find only through links, while the other briefs became host skills in PR #3. It is also the GEPA seed (`experiments/gepa/README.md`), so its body must stay usable as the optimization candidate.

## Scope
`xtask/src/router.rs`, `xtask/src/main.rs` (verdict arguments), `xtask/tests/router.rs`, `xtask/src/checks.rs` if a parity check is needed, `.claude/skills/investigate/` and `.agents/skills/investigate/`, docs (ARTIFACTS.md, README, experiments docs), `harness/readiness.json`, `harness/docs.json`.
Tier: standard. It touches protected xtask paths and the ceremony's skill folders.

## Acceptance as named tests
- `commit_step_names_the_page_whose_row_is_uncommitted`: given blindspots passed and committed and interventions passed with the log uncommitted, `next` says commit interventions.html.
- `verdict_records_findings_and_model`: given `--findings N --model M`, the row stores both, and old rows without them still parse.
- `investigate_skill_matches_its_seed`: given the two host copies and docs/skills/investigate.md, the skill body equals the seed, so GEPA and the hosts read the same text.

## Validation
`cargo xtask check ci > target/verification/ci.json`; `cargo xtask complete --task router-polish --base origin/main --report target/verification/ci.json`; `cargo xtask task-artifact next --task router-polish` reads `complete`.

## Out of scope
Making completion read the log; confining Bash writes to the log; changing GEPA's evaluator or the seed's wording.
