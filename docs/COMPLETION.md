# Task completion and engineering gaps

The harness checks evidence and reconciliation. It does not understand ticket
workflows, generate gaps, decide priorities, or prove every defect was discovered.
Tasks can come from a prompt, plan, Jira, Linear or GitHub Issues. An optional
work_item string is an opaque reference; it is never fetched or interpreted.

## Three different scopes

| Record | Purpose |
| --- | --- |
| Task / ticket | The currently assigned outcome, scope and coordination |
| Project gap | A known engineering shortcoming or release blocker |
| Requirement | Behavior that evidence must establish |

harness/gaps.json is a focused project engineering register, not a copy of your
backlog. `cargo xtask gaps` only reads, validates and displays it. Open gaps outside
the assigned work do not prevent task completion. `release <scope>` checks every
requirement and open blocker in that release scope.

## Where gaps come from

Checks expose failed tests, missing or stale evidence and structural violations.
Agents investigate those signals, exercise workflows and compare behavior with
requirements. They may also discover unsupported assumptions, missing recovery,
unverified platforms or misleading documentation. A failing command is a signal
to investigate, not an automatically created gap.

Record a gap when an observed, material engineering shortcoming remains unresolved
and matters beyond the transient edit loop. Include evidence/reproducer, impact,
next action, priority and any justified blocking scope. Link executable requirements
where possible. Label uncertain findings as an investigation, with an explicit
next action; do not present speculation as a confirmed defect.

A compiler error introduced and fixed during the task needs no gap. A known
persistence defect left unresolved does. A planned feature is ordinary backlog
work unless it is required for an acceptance or release claim. Merge duplicate
findings into existing gaps. No quota requires a new gap per task.

## Checkpoints

1. Task start: read relevant requirements and the project gaps. Identify which
   gaps the assignment addresses and create harness/tasks/ID.json using
   examples/task-record.json. A ticket URL, ID or plan reference is optional. Before
   implementation, write and commit its five discovery pages; commit notes during the
   work and the buy-in doc and quiz last ([task artifacts](ARTIFACTS.md)).
2. Material discovery: investigate immediately. Fix it within authorized scope,
   update an existing gap, or record a new durable gap/observation. Do not silently
   expand the task into a project-wide audit.
3. Milestone: update evidence, remaining actions and justified gap dispositions.
4. Before stopping: answer the four review questions below, reconcile discoveries
   and docs, then run the appropriate completion/handoff command.

The completion gate checks this bounded review; the default stop hook runs it when
a record claims completion. It does not launch a model or an audit.
An explicitly requested bounded review is the right way to search more broadly;
see [the maintenance procedure](MAINTENANCE.md).

## Task review and record contract

Schema version 6 requires the [HTML task artifacts](ARTIFACTS.md) in addition to
review, with four nonempty answers:

- acceptance: Did any assigned acceptance criterion remain unmet? Explain the
  outcome and remaining work; unmet assigned acceptance means partial or blocked.
- verification: Did checks expose a pre-existing or unresolved failure? Record
  durable findings. Transient failures already fixed need no new gap.
- assumptions: Did the work rely on an assumption you could not verify? State
  the supporting evidence or where the unresolved assumption was recorded.
- coverage: Did you find a material missing test, recovery path or platform check?
  Record it, or explain why none was found in the inspected scope.

Blank answers are allowed while drafting but block handoff and completion.
The gate returns these questions as actionable diagnostics. It checks that answers
exist; reviewers assess whether the answers are honest, relevant and sufficient.
“No material gaps discovered in the inspected scope” is valid after review.

The other fields are demonstrated in [the example](../examples/task-record.json):

- requirements names registered acceptance IDs. Complete requires current passing
  full-CI evidence and verified requirements that actually represent the task.
  When the task adds tests that prove a requirement, list them under that
  requirement in `harness/readiness.json` in the same change; otherwise the
  requirement verifies against its old tests alone.
- related_gaps includes rationale and disposition. resolved requires a closed gap
  and verified linked requirements. updated means changed against base; unaffected
  means unchanged. retired means removed with a matching transition receipt.
- gap_review explains which gaps are relevant, including when none apply.
- discoveries records a summary and a durable source file, HTTPS reference, or
  gap:ID. Use gaps for unresolved engineering shortcomings; documents can record
  resolved observations or reasoning. The agent writes these records, not the hook.
- discovery_review summarizes findings or explicitly records none discovered.
- documentation.updated lists existing source files actually changed from base;
  rationale explains updates or why none are necessary.
- summary and remaining form the handoff. Partial/blocked status needs next steps.
  Complete may retain follow-up work outside its accepted scope.

References are not proof that a remote document exists or its contents are true.
Unknown fields are rejected. Archived v1–v5 task records remain readable; upgrade any
active record making a new handoff/completion claim to v6, write its
[artifacts](ARTIFACTS.md) and answer the review. Git history preserves earlier page
versions; artifact-only edits also trigger reconciliation.

## Commands and source-bound evidence

Finalize task/gap records before CI. They are part of source identity; changing
them invalidates earlier evidence. Put generated reports under target.

```sh
BASE=origin/main
cargo xtask gaps
cargo xtask gaps --base "$BASE"
# Review required_changes and author any required transition receipt.
mkdir -p target/verification
cargo xtask check ci > target/verification/ci.json
cargo xtask complete --task TASK-142 --base "$BASE" --report target/verification/ci.json
cargo xtask handoff --base "$BASE" --report target/verification/ci.json
```

complete gates one assigned task. handoff checks all added/modified task records,
requires one for a changed tree, accounts for all changed gaps, and validates
transition receipts. It permits partial/blocked records with explicit remaining
work; all_tasks_complete distinguishes reconciliation from completion. Old task
acceptance is not rerun as a new claim on every change. Keep task history.

## Consequential transitions

`gaps --base REV` compares the working tree (including untracked files) with the
Git merge base of REV and HEAD. It detects deleted gaps, closures, removed blocking
scopes or requirement links, closed-gap resolution/requirement edits, and newly
introduced closed gaps. Ordinary priority/evidence/next-action updates need no
transition receipt. Missing Git history fails; it is not silently ignored.

The output supplies base_commit and required_changes. Copy exact change objects
into a uniquely named harness/transitions/ID.json using
[the receipt example](../examples/transition-receipt.json). Supply specific rationale
and evidence. Hashes bind explanations to the exact changes, not a later revision.
Linked closures need verified requirements; manual closures need an explicit
manual_justification. Keep closed gaps. Deletion/supersession needs rationale and
an appropriate successor reference. Never auto-approve generated change hashes.

The policy subject covers config, readiness, scenarios, workflows, goldens, xtask,
scripts, CI, Cargo manifests, agent instructions and core acceptance/closure docs.
The output lists protected_files. Existing receipts at the comparison base are
immutable; add new records for later changes. Recompute and review unmerged active
receipts after edits or rebasing. A receipt explains intent, not reviewer approval.

## Default hooks and CI

The repository ships project hooks for Claude Code (`.claude/settings.json`) and
Codex (`.codex/hooks.json`, `.codex/rules/default.rules`). They move checks that
CI would otherwise make at the PR to the moment an agent acts:

- **Before an edit or shell command**, [scripts/agent-hook.sh](../scripts/agent-hook.sh)
  `guard` reads the event without running Cargo. Golden baselines and the
  dependency allowlist in `harness/config.json` ask on Claude Code and are refused
  on Codex, which cannot ask. Other protected policy files pass with a note that the
  branch needs a transition receipt. A PR merge asks on Claude Code and is refused
  on Codex; a force-push is refused on both. Shell
  commands count only write destinations, so a commit message or fixture that
  mentions a golden passes; a script that builds the path itself is caught only
  at completion and CI.
- **After an edit**, `check` runs `cargo check --locked` and `cargo fmt --check`
  on the changed crate (the workspace for a manifest or `harness/config.json`),
  then the built xtask's `architecture` check. Failures come back to the agent.
- **At stop**, [scripts/agent-complete.sh](../scripts/agent-complete.sh) gates
  completion claims only. A turn that claims nothing stops. When a task record
  changed on this branch says `"status": "complete"`, `cargo xtask complete`
  judges it and a refusal keeps the agent working; setting the record back to
  partial with its remaining actions always clears the block. `HARNESS_BASE`
  (default `origin/main`) and `HARNESS_REPORT` (default
  `target/verification/ci.json`) still apply; `HARNESS_TASK_ID` no longer picks
  the task. Partial handoffs, stray files and policy receipts surface at PR CI.

Command rules ask before `cargo add`, `cargo remove` and `gh pr merge`, and deny
the literal force-push spellings (Codex forbids merges and force-pushes). They are speed bumps: the guard catches other
spellings, and only hosting rules can stop a merge.

Requirements and limits: an error inside the guard script exits 2 and blocks on
both hosts. A guard that cannot start or runs past its 10 s timeout blocks on
Claude Code only with `onFailure`, which needs v2.1.295 or later; older versions,
and Codex, let that call through. Codex runs project hooks only after
the project's `.codex/` layer is trusted and each hook is reviewed in `/hooks`. Hooks
never run in CI, which still runs `cargo xtask check ci` and, on pull requests,
`handoff`. To turn hooks off for a session, set `"disableAllHooks": true` in
`.claude/settings.local.json` or run `claude --settings '{"disableAllHooks": true}'`;
on Codex set `[features] hooks = false`. `.claude/` and `.codex/` are protected
paths, so changing a hook on a branch needs a transition receipt. Claude Code's
per-machine `.claude/settings.local.json` and `.claude/worktrees/` are ignored and
never change the source fingerprint. User cancellation stays possible: neither
host runs the stop gate after an interrupt.

The discovery-page guard stays opt-in: [the Codex example](../examples/codex-artifact-hooks.json)
blocks `apply_patch` edits to product code until a task's five discovery pages
are committed when `HARNESS_TASK_ID` is set.

PR CI fetches full history and passes the event base SHA through an environment
variable. It checks handoff against the tested checkout; ordinary checks stay
usable with open product gaps. Push/scheduled jobs do not reinterpret historical
tasks as new work. A reconciled partial PR is not a product release claim.

Configure required status checks and independent review of acceptance, policy,
baseline, workflow, hook and closure changes in hosting settings. Assign real
CODEOWNERS. Local hooks and self-authored receipts cannot enforce that authority.
PR code can modify its verifier; use a trusted verifier for adversarial contributions.

## Optional tracker integration and migration

The core has no snapshot, provider, status or freshness contract. Teams may build
an integration outside the normal harness if they deliberately want tracker-backed
gap maintenance. Keep one gap authority, preserve provenance and validate exports
in that integration; do not silently maintain two competing backlogs. The stock
starter's project register remains repository-owned, with optional opaque links.

From the earlier snapshot-based ZIP: remove the unused repository-mode
harness/tracking.json and tracker examples. If external mode was actually enabled,
reconcile the authoritative records into local gaps with reviewed resolution files
before switching; the harness rejects that legacy mode rather than trusting a
stale local copy. Historical external-authority comparisons require a deliberate
migration baseline; there is no automatic remote migration or completeness claim.

## Artifact admission

Completion requires the task's eight pages, each carrying the shared stylesheet
inline, and reads the branch's own commits since the base (merge commits skipped): the five
discovery pages before the first product-code commit, implementation notes before
the last one, and buy-in and quiz at or after it. Product code and the task's pages
must be committed; task records, gaps and reports need not be. Commit order shows
sequence, not usefulness, and a rewritten history can fake it. An independent
reviewer agent and human review against the exemplars judge quality. The default stop hook invokes the same completion checks for a claimed task; the
opt-in discovery-page guard is separate.
