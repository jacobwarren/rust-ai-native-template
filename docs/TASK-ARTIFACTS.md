# Mandatory HTML task reviews

**Default: [author task-specific HTML](AUTHORED-HTML.md) and register its purpose
and provenance.** The filenames, structured models and exact rendering described
below belong to the optional fixed starter workflow. They are not constraints on
authored HTML. Both modes retain the required discovery and lifecycle purposes.

Every active task has a checked-in HTML review collection under
`harness/task-artifacts/ID/`, backed by one authoritative `record.json`:

- `unknowns.html`: index of the [five mandatory discovery tools](DISCOVERY.md), with the preserved initial preflight.
- `implementation.html`: accumulating implementation notes.
- `buy-in.html`: reviewer brief, demonstration, evidence and tradeoffs.
- `quiz.html`: interactive scenario quiz and explanations.

`index.html` is a small linked overview, not a combined scrolling document. All
ten HTML files are required, including for small tasks. Keep content proportional
to the work. The start command creates the lifecycle shell; discover creates the five required tools before implementation. Pending later stages are visibly labeled.

## Lifecycle commands

Create the task record using [the example](../examples/task-record.json) with
schema_version 4, then author inputs using these examples. Inputs may live under
target; their content is copied into the durable record. Run from the repository:

```sh
cargo xtask task-artifact start --task TASK-142 --input examples/task-artifact-before.json
cargo xtask task-artifact discover --task TASK-142 --input examples/task-discovery.json
cargo xtask task-artifact note --task TASK-142 --input examples/task-artifact-note.json
cargo xtask task-artifact publish --task TASK-142 --input examples/task-artifact-after.json
cargo xtask task-artifact check --task TASK-142
```

Replace the example text with actual task-specific reasoning. Open the returned
index or any of the standalone HTML paths in a browser. No server, package installation, model call or network
connection is needed. Relative evidence links work with the repository alongside
the page; HTTPS evidence links need connectivity when followed. Each HTML file contains its own styling and, for the quiz, its script. Copying a
page preserves that content but not sibling navigation or local evidence files.

1. **Before code:** run start after inspecting relevant contracts. Describe the
   outcome, unknowns, assumptions, alternatives, independent expected result and
   verification boundary. Start refuses to replace an existing snapshot. Use
   blindspot analysis, reference comparisons, mock sessions or human questions
   when useful; do not manufacture unknowns or block routine decisions.
2. **During work:** run note at material discoveries and milestones. Record the
   observation or changed assumption, decision/rationale, user-visible effect and
   evidence references. Explicitly state when no material deviations occurred.
   Append corrections instead of editing the initial snapshot or previous notes.
3. **After implementation:** publish a balanced buy-in brief: what changed and
   why, a before/after demonstration, verification and its limits, risks/tradeoffs
   and decisions needed. Label simulations, expected outcomes, observed results
   and unrun checks accurately. Publishing retains earlier brief revisions.
4. **Before merge:** include at least one meaningful scenario question with two
   or more distinct choices, a zero-based correct answer, explanatory feedback,
   and evidence references. The quiz artifact lets humans choose, check and retry answers.
   A quiz is mandatory content; passing it never means approved or ready to merge.

A later note makes the previous publication out of date; publish an updated brief
and quiz before completing the task. Rendering alone cannot mark it current.

## Admission and migration

New/modified tasks must use v4 for handoff, complete and the opt-in stop hook.
Unchanged historical v1/v2/v3 records remain readable; there is no fabricated
preflight requirement for archived history. When resuming an old task, create an
honestly labeled current snapshot and explain that earlier work preceded it.

Coordination checks v3/v4 artifacts, including exact rendered bytes. Partial/blocked
handoffs need a valid initial record and all ten HTML files and explicit remaining actions, but can
leave later stages pending. Blocking unresolved interview answers prevent completion. Complete needs at least one implementation note and
a valid buy-in brief/quiz covering the latest note, plus the existing review,
registered acceptance and fresh full-CI evidence. No stage can be waived for a
complete task. Artifact-only edits also trigger task reconciliation.

The admission comparison preserves the initial snapshot and previously committed
notes and brief revisions. New entries append. The authoring commands preserve
uncommitted entries too, but a determined direct file editor can rewrite history
before it reaches Git. These are local review controls, not signed attestations
of chronology, human comprehension, identity or correctness.

## Recovery, evidence and ownership

The JSON record is authoritative. The lifecycle pages, two indexes and five
discovery tools are deterministic mandatory presentations.
If a write is interrupted between JSON and the HTML replacements, validation fails.
Repair the HTML without changing reasoning using:

```sh
cargo xtask task-artifact render --task TASK-142
```

Mutations take an exclusive per-artifact `.lock`. An interrupted process may leave
it behind. Confirm no writer is active before removing that lock and rendering.
Start never overwrites an existing record. Inputs are strictly parsed before
publication; unsafe output IDs, symlink artifact paths and invalid references
are rejected. The renderer escapes all authored content and uses fixed offline
CSS/JavaScript. It does not run task-provided HTML, scripts or Markdown.

Finalize task records and rendered HTML before CI. They participate in source
identity. The provenance disclosure on each page shows the source identity captured at start, not a claimed
final verification fingerprint. Keep final CI and completion reports under target;
link them in the PR. Do not edit a checked-in brief to insert its own final CI
fingerprint and thereby invalidate that report. A plan or PR can link the index or individual pages
rather than duplicate its narrative. Product docs remain authoritative for the
current feature contract; task pages explain changes at particular revisions.

## Implementation and verification boundary

The CLI is in xtask/src/task_artifact.rs; the fixed renderer is task_view.rs and
task_view.html, with fixed feedback in task_quiz.js. Public CLI tests exercise lifecycle, malformed input, escaped
content, missing/stale HTML, preserved history, legacy admission and completion.
Automated structure checks do not assess whether explanations are useful or
correct. Review the actual page, questions, demonstrations and evidence as a human.

An additional optional check is `node scripts/test-task-quiz.cjs`. It exercises
the actual quiz handler with a stub DOM, including unanswered, wrong, correct and
retry feedback. It does not verify browser layout, CSP enforcement or keyboard
interaction. Node is not required by the normal Rust CI profiles.
