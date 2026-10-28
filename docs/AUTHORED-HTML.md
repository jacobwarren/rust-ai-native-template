# Author HTML for the task

HTML is the default human review medium, not a fixed form to fill out. Generate
purpose-built HTML/CSS/SVG/JavaScript from the actual code, evidence and decisions.
Choose layouts, diagrams, simulations, comparisons, editors and interactions that
make this particular problem understandable. Use the [shared styling foundation](ARTIFACT-STYLE.md) for consistent presentation.
Starter components are optional; exact templates and fixed interaction or feedback schemas are not
required. The agent authors files directly with normal file tools.

## Required purposes, flexible presentation

Before implementation, provide separate HTML artifacts for blindspots,
interventions, interview, reference semantics and the tweakable plan. Keep them
proportional to the task. Name the inspected scope and evidence; do not invent
findings, questions or reference code. A settled interview or missing reference
needs an honest explanation and appropriate remaining actions, not fabricated data.

During implementation, maintain notes. Before completion, provide a buy-in brief
and a comprehension quiz. Supporting pages and assets are welcome. Filenames,
layouts, DOM structure, graphs, code excerpts, controls and response payloads are
chosen by the author. Use additional files when that better explains the feature.

Every artifact should give the human a useful way to respond: copy as prompt,
copy JSON, download selections, export a diff, annotate a comparison, or simply
identify the claim that needs correction. Choose the mechanism for the problem.
No browser selection, quiz score or declared actor is approval by itself.

## Register a reviewable revision

Create the normal v4 task record. Run `cargo xtask task-artifact style --task ID`
to install the shared foundation, then author files under harness/task-artifacts/ID/.
Each HTML head must link its local foundation.css as documented in the style guide.
Then list their purposes and review context in a small registration input:

```sh
cargo xtask task-artifact register --task ID --input PATH-TO-MANIFEST
cargo xtask task-artifact check --task ID
cargo xtask task-artifact context --task ID
```

See [the manifest example](../examples/authored-artifacts.json). Each entry has:

- purpose: blindspots, interventions, interview, reference, plan, implementation,
  buy-in, quiz, or supporting. Required purposes have distinct HTML files.
- path: a relative path inside this task's artifact directory.
- description: what the human can learn or decide with this artifact.
- evidence: relevant repository source files or HTTPS references.
- feedback: how the human returns reactions or corrections.
- verification: checks actually performed and their limits, including unrun checks.

This metadata describes the contract; it does not define the artifact's content
model. Supporting assets can be CSS, JavaScript, SVG, data, images or additional
HTML. Register the complete bundle, including assets needed to reproduce it.
Prefer self-contained files and local bundled assets over runtime dependencies.
The harness does not crawl dependencies or prove that external resources exist.

Registration automatically includes the installed foundation.css in the bundle.
Do not list that managed asset in the input manifest. Registration saves authored.json and an immutable byte snapshot under
`authored-revisions/N/`, preserving relative relationships within the bundle.
Evidence links outside that bundle may still require the original repository.
Each new registration appends a revision. Its hashes record the bytes you chose;
they are not hashes of a prescribed template. Edited files need a new registration.
Missing files, changed snapshot bytes and rewritten committed history fail checks.
Partial handoffs need the five discovery purposes; completion also needs notes,
buy-in and quiz plus the existing acceptance and current full-CI evidence.

## Flexible feedback with bounded provenance

Use copied text directly in the conversation, or import a durable response through
`task-artifact decide`. The transport envelope records provenance; payload can be
any non-null JSON value, including a plain string, custom nested objects, selected
geometry, sliders, reordered items or a textual diff. There are no fixed response
keys or enumerated choices imposed by the harness.

```json
{
  "schema_version": 1,
  "task": "TASK-ID",
  "revision_sha256": "HASH-FROM-CONTEXT",
  "feedback_base_sha256": "HASH-FROM-CONTEXT",
  "actor": "reviewer name or role",
  "payload": {"anything_task_specific": "the exported response"}
}
```

```sh
cargo xtask task-artifact decide --task ID --input RESPONSE-ENVELOPE.json
```

Use context for the exact revision the human reviewed; never substitute current
hashes to make a stale response pass. A page can export just its task-specific
payload for the agent to wrap after identifying that reviewed revision. Do not
embed the page's own final hash into its bytes: that would be self-referential.
The import checks task identity and revision/feedback hashes, appends history and
never executes the payload. Actor labels are declarations, not authentication.

Import does not regenerate or overwrite HTML. Reconcile the response into the
artifacts and register a new reviewed revision before completion. The agent still
must implement consequential decisions; the gate cannot judge semantic alignment.

## Optional fixed starters and migration

The earlier start/discover/note/publish/render commands and structured models are
optional starter tools, not the default authoring workflow. They remain available
for legacy records or when their presentation fits. Agents must not choose them
merely because the files exist. The richer five-purpose guidance in
[discovery](DISCOVERY.md) informs authoring; it does not prescribe controls.

To customize an existing generated task, edit/copy its HTML or author replacements,
then register a complete bundle. Keep record.json and preceding history. Once
registered, template mutation/render commands refuse to overwrite authored HTML.
Use normal file editing and register instead. Historical fixed-mode tasks are not
silently migrated or rewritten. Deleting a committed authored ledger cannot
silently downgrade a task back to fixed-mode admission.

Registration uses an exclusive authored.lock and atomic ledger replacement.
Interrupted registration can leave an orphan authored-revisions/N directory or
lock. Confirm no writer is active, inspect/preserve useful bytes, then remove only
the interrupted unregistered output before retrying. Do not regenerate arbitrary
HTML as a recovery shortcut. Finalize source, manifests and snapshots before CI.

## What checks do and do not establish

The registry reads files; it never runs their JavaScript. Checks validate purpose
coverage, paths, metadata, registered bytes, snapshot history and feedback
provenance. They do not certify useful content, safe JavaScript, accessibility,
correct simulations, complete source evidence or visual quality. Inspect and test
interactions appropriate to each artifact and record the boundary reached.
Custom HTML is authored code, not an automatically sandboxed document.

Independent CLI tests demonstrate that different SVG/canvas/script layouts pass
unchanged, custom payloads round-trip, and missing purposes, unsafe paths,
symlinks, unregistered edits, stale imports and rewritten snapshots fail. The
existing optional renderer tests remain tests of those starters, not constraints
on authored documents.
