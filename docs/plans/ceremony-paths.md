# ceremony-paths

Written with the write-ticket skill (.claude/skills/write-ticket/SKILL.md).

## Problem
A small task cannot touch any skill. Two rules stack:

- `xtask/src/transitions.rs:39-47` protects every path under `.claude/` and
  `.codex/`, so any edit there needs a policy receipt and forces a standard
  task (`size::forces_standard` calls `transitions::protected`).
- `xtask/src/size.rs:12-19` also counts all of `docs/skills/`,
  `.agents/skills/` and `.claude/skills/` as ceremony.

Fixing a typo in the investigate skill, or adding a new skill unrelated to the
task pages, therefore costs eight pages, sixteen subagent runs and a receipt.
The only skills that define the ceremony are author-artifact, review-artifact
and write-ticket. The investigate and add-product-command skills do not.
Narrowing only size.rs would free nothing under `.claude/skills/`, because the
protection in transitions.rs still applies.

## Scope
- `xtask/src/transitions.rs`: `.claude/` stays protected except
  `.claude/skills/<name>/` for a skill outside the ceremony.
- `scripts/agent-hook.sh`: the guard copy of that rule.
- `xtask/src/size.rs`: ceremony becomes the three ceremony skills in both
  hosts, their two `docs/skills/` pointers, `docs/ARTIFACTS.md` and the agent
  folders.
- Tests in `xtask/tests/tiers.rs`, `xtask/tests/hooks_guard.rs` and
  `transitions::tests`.
- Docs: `docs/ARTIFACTS.md` "Task size", `docs/COMPLETION.md` "Default hooks
  and CI", and the write-ticket skill in both hosts.
- `harness/readiness.json` test names and a policy receipt.

Tier: standard. These are policy and ceremony paths.

## Acceptance as named tests
- `unrelated_skills_stay_small`: given a small task that edits the investigate
  skill in both hosts and its seed `docs/skills/investigate.md`, when handoff
  runs, then the tier holds and no transition receipt is required.
- `protected_cargo_and_ceremony_paths_force_standard` (extended): given an edit
  to any ceremony skill in either host, a `docs/skills/` pointer,
  `.claude/settings.json`, an agent file or a new non-skill file under
  `.claude/`, then a small task is refused.
- `guard_passes_unrelated_skills_without_a_receipt_note`: given an edit to
  `.claude/skills/investigate/SKILL.md`, the guard passes without a policy
  note; an edit to `.claude/skills/author-artifact/SKILL.md` still gets one.
- `transitions::tests::guard_tiers_match_protected_paths` (extended): the
  guard's ceremony-skill list equals the one in transitions.rs.

## Validation
`cargo xtask check ci > target/verification/ci.json`;
`cargo xtask gaps --base origin/main` lists the protected changes for the
receipt;
`cargo xtask complete --task ceremony-paths --base origin/main --report target/verification/ci.json`.

## Out of scope
- The 100-line limit and the administrative paths.
- What is protected under `.codex/`, which holds no skills.
- A rewrite-rate report from `reviews.json`.
- Hosting rules, which the hosting-enforcement gap covers.
