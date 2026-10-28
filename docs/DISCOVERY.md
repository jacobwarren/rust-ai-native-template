# Interactive task discovery

**Default: freely [author HTML for the task](AUTHORED-HTML.md).** The five purposes
are required; their layouts, content models, interactions and response formats are
not fixed. The commands, typed data and controls below document optional starters.
Use the [authoring quality guide](ARTIFACT-AUTHORING.md) and
[worked studies](../examples/artifact-studies/index.html) for the expected human
utility. A starter's existence or successful rendering is not a reason to choose
its layout or a substitute for substantive task-specific reasoning.

Every active v4 task needs five separate HTML tools before implementation. The
[task lifecycle](TASK-ARTIFACTS.md) keeps its initial preflight and final artifacts;
unknowns.html now links the five tools rather than reducing them to prose cards.
The three other source techniques (domain explainer, visual directions and mock)
remain optional future extensions, not requirements or implemented modules.

## Author and explore

After creating the task record and running task-artifact start, author structured
content using [the example](../examples/task-discovery.json), then run:

```sh
cargo xtask task-artifact discover --task ID --input PATH-TO-DISCOVERY-JSON
```

The command appends a discovery revision and writes these self-contained pages:

| File | Purpose and interactions |
| --- | --- |
| blindspots.html | Evidence-backed traps, consequences and constraints; include, investigate or exclude findings to assemble the response |
| interventions.html | Code-grounded options with relative effort, benefit and tradeoffs; select what resonates or leave it aside |
| interview.html | Consequential questions one at a time; alternatives, custom answers, progress, previous/next and editable decision summary |
| reference.html | Paired source/target semantics with linked hover/focus highlights, differences, expected outcomes and correction controls |
| plan.html | Consequential alternatives with changing interface/behavior previews; execution order and collapsed mechanical work |

The starter provides these fixed interactions. Authored mode instead accepts
task-specific HTML/CSS/SVG/JavaScript and arbitrary feedback payloads. All five
purposes remain mandatory; depth and presentation scale with the task.

Scope the blindspot pass and link actual evidence. An empty findings list needs
an explicit no-findings explanation. Compare at least two real interventions;
leaving behavior unchanged can be a meaningful alternative. Do not manufacture
an unrelated second feature merely to fill the list.

Order interview questions by consequence. An existing answer must include its
basis (for example, an explicit user instruction); it is displayed as an existing
answer, not presented as newly supplied human approval. Empty interviews need a
no-questions explanation. Blocking questions without sourced answers or imported
responses prevent completion. Nonblocking exploratory questions may remain open.

A reference can be a local implementation, an external source, or an explicit
contract. Name its revision, source, proposed target, mapped behavior, differences
and independent expected outcomes. If no suitable reference exists, retain the
artifact with an explicit missing-reference explanation; never invent excerpts.
That structural exception does not authorize an agent to proceed past a genuinely
blocking unknown. Record remaining actions in an honest partial task.

A plan must expose at least one consequential choice, meaningful alternatives,
a labeled recommendation, previews and execution steps. Recommendations preview
by default, but are not exported as human decisions unless explicitly selected.

## Export and record feedback

Interactions update a page-local draft. Session storage is used where supported
and keyed by task, page, discovery revision and feedback revision. Browsers that
restrict storage still support interaction and export, but may lose unsaved drafts
on reload. Use Copy summary to discuss decisions, or enter your name/role and
Export decisions to download the JSON proposal.

```sh
cargo xtask task-artifact decide --task ID --input PATH-TO-EXPORTED-JSON
```

This command validates task identity, the current discovery hash, the current
feedback-history hash, response keys and option values. It appends feedback and
regenerates the pages. Reopen the regenerated pages after each import: exports
from older tabs fail explicitly, even if their responses concern another section.
This conservative rule prevents silent overwrites; it is not an automatic merge.

Imports are patches: keys not present retain prior responses for that discovery.
A subsequent response for the same key supersedes its effective value while
preserving the earlier entry. A new discovery revision starts a new decision
context; prior responses remain in history but are not silently carried forward.
Interview answers and reference corrections accept free text; plan selections
must identify an actual option. No response executes a command or grants approval.
Actor labels are declarations, not authenticated identities.

Draft interactions do not change source files. Importing does, invalidating prior
CI and the final brief's input checkpoint. Republish the buy-in brief/quiz after
new discovery or feedback, finalize source, then rerun CI. The plan page reflects
imported selections; the agent must still reconcile their consequences into code,
acceptance and execution steps. The harness does not autonomously implement them.

## Versioning and recovery

Active task schema v4 requires all five tools. Historical v1–v3 tasks remain
readable, but need explicit migration for a new admission claim. A resumed task's
discovery must be labeled as current or retrospective; never backdate it.

Artifact schema v2 adds append-only discoveries and feedback to the retained
preflight, notes and publication history. The discover command upgrades a v1
artifact without rewriting the preflight. Existing published briefs may become
stale; publish a new revision rather than altering the old one.

Rendering uses ten HTML files: task index, discovery index, five tools, notes,
buy-in and quiz. Missing or stale files fail checks. Interrupted generation is
repairable with `cargo xtask task-artifact render --task ID`. The existing lock
and source identity rules still apply. Failed validation/import leaves the
record unchanged. Git comparisons reject rewriting committed discovery or
feedback prefixes. These local controls cannot establish when uncommitted work
was actually performed or authenticate a purported human answer.

## Verification

Rust CLI regressions in xtask/tests/discovery.rs cover the five required pages,
strict models, empty-state explanations, escaping, stale/wrong-task/unknown-choice
feedback, append-only history, unresolved blocking answers and stale publications.
Normal Rust CI has no Node dependency.

Optional UI integration uses the actual generated scripts with a DOM implementation
and then imports their exports through the real CLI in an isolated test workspace:

```sh
cargo build -p xtask --locked
npm install --prefix target/discovery-ui --no-audit --no-fund --package-lock=false jsdom@30.1.3
NODE_PATH="$PWD/target/discovery-ui/node_modules" node scripts/test-discovery-ui.cjs
```

That check exercises all five UIs, custom answer editing, linked highlighting,
changing previews, export/import round trips and reopening imported decisions.
It does not verify browser geometry, CSP enforcement, native clipboard/download
permissions or visual quality. Review those in a browser; do not relabel DOM
execution as visual verification.
