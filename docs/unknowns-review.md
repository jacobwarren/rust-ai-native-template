# Proposal: task explanations and unknown discovery

Status: historical assessment, superseded by mandatory HTML policy. Reviewed: 2026-10-28.

The user subsequently required HTML for every active task. The implemented
contract is [Mandatory HTML task reviews](TASK-ARTIFACTS.md). Recommendations
below preserve the initial review, including the rejected optional-HTML approach.
Task: unknowns-review. Inspected base: f78c3c15e89902d3071308c56e4243dc4054c16e.

## Recommendation

Adopt a small, evidence-linked explanation across each task's lifecycle.
Use richer discovery techniques when the task warrants them. Do not require
all eleven artifacts, HTML output, a model service, or a human quiz for every task.

The intended benefit is a durable account of the original uncertainty, choices,
changed assumptions, observable outcome, and remaining limits. That can help
reviewers understand features and future maintainers reconstruct intent. This
benefit is plausible, but the examples do not establish measured effectiveness.
Pilot the workflow before enforcing new gates.

Three delegated reviews examined the pre-, during-, and post-implementation
prompts and example artifacts at the [source index](https://thariqs.github.io/html-effectiveness/unknowns/index.html).
This document records their synthesis and repository-specific implementation proposal.
No runtime or completion policy has been implemented by this assessment.

## Prompt-by-prompt disposition

These are adaptations for this repository, not requirements imposed by the source.

| Source prompt | Use when | Proposed durable output |
| --- | --- | --- |
| [Blindspot pass](https://thariqs.github.io/html-effectiveness/unknowns/01-blindspot-pass.html) | Unfamiliar implementation, persistence, security, migration, or consequential behavior | Plan Context/Discoveries: inspected scope, evidence, uncertainty, resulting constraints |
| [Teach me my unknowns](https://thariqs.github.io/html-effectiveness/unknowns/02-color-grading-explainer.html) | Builder or reviewer needs domain vocabulary | Optional explanation; reusable terms belong in product/architecture docs |
| [Four design directions](https://thariqs.github.io/html-effectiveness/unknowns/03-design-directions.html) | Visual direction or interaction design is unresolved | Bounded alternatives and selected rationale in Decisions; do not require four options |
| [Mock before you wire](https://thariqs.github.io/html-effectiveness/unknowns/04-toolbar-mock.html) | Interaction is ambiguous or integration is expensive | Labeled prototype; for this CLI, sample command sessions and expected JSON may suffice |
| [Brainstorm the intervention](https://thariqs.github.io/html-effectiveness/unknowns/05-churn-brainstorm.html) | Outcome is known but solution is uncertain | Code-grounded option comparison; distinguish observed friction from hypotheses about user impact |
| [The interview](https://thariqs.github.io/html-effectiveness/unknowns/06-interview.html) | Missing answers materially affect acceptance, architecture, cost, or authority | Decisions and remaining questions, ordered by consequence; avoid routine permission requests |
| [Point at a reference](https://thariqs.github.io/html-effectiveness/unknowns/07-reference-port.html) | Porting, compatibility, or adapting a reference | Reference revision, preserved/changed semantics, edge cases, independent expectations and tests |
| [The tweakable plan](https://thariqs.github.io/html-effectiveness/unknowns/08-implementation-plan.html) | Existing multi-step/risky plan is required | Consequential choices near the top of the existing plan, with execution milestones retained |
| [Implementation notes](https://thariqs.github.io/html-effectiveness/unknowns/09-implementation-notes.html) | Every task, proportionate to material discoveries | Original assumption, observation, decision, consequence and verification; explicit none when appropriate |
| [The buy-in doc](https://thariqs.github.io/html-effectiveness/unknowns/10-pitch-doc.html) | Meaningful behavior or architecture change | Neutral reviewer brief with before/after behavior, evidence, limits and decisions needed |
| [Quiz me before I merge](https://thariqs.github.io/html-effectiveness/unknowns/11-change-quiz.html) | Optional learning aid for unfamiliar or consequential changes | A few scenario questions with evidence-backed explanations; no merge authorization or readiness score |

## Existing support and the actual missing piece

The [plan template](plans/TEMPLATE.md) already stores context, discoveries,
decisions, interfaces, recovery, evidence and outcomes. Its [restart contract](plans/README.md)
already requires explicit deviations and source identity checks.
The [task contract](COMPLETION.md) already reconciles discoveries, assumptions,
coverage and acceptance at task start, discoveries, milestones and handoff.
The [PR template](../.github/pull_request_template.md) already asks for outcome,
change, evidence, risk and reconciliation.

The missing piece is a consistent reader-facing connection between these facts:
what was uncertain before work, what evidence changed the approach, and what a
human should understand afterward. Adding another backlog or parallel plan would
make that connection harder to maintain.

The [task parser](../xtask/src/task_record.rs) rejects unknown fields. The
[completion gate](../xtask/src/completion.rs) checks references and review presence,
not the quality of reasoning. Existing discoveries can reference durable local
source files, HTTPS references, or gaps. Do not append arbitrary JSON fields or
misuse discoveries to invent a finding for every task.

## Proposed lifecycle

### Before implementation

For every task, briefly state the outcome, independent expected result, public
boundary, inspected contracts and important unknowns. Select a discovery technique
only if it resolves a relevant uncertainty. State why no expansion is needed
for routine work. For planned work, put this in Context and Decisions; for small
work, keep it in the task description or a short linked note.

Suggested prompt:

> Inspect the task, relevant contracts, implementation, acceptance and gaps.
> Identify material unknowns and separate observations from hypotheses. Choose
> only the discovery techniques that could change this task's approach. Record
> decisions, evidence and unresolved questions in the existing task or plan.
> Ask the human only for missing information that materially changes acceptance,
> architecture or authority. Define an independent expected outcome and the
> public boundary that will establish it before implementing behavior.

### During implementation

Keep material deviations in the existing Discoveries/Decisions sections. A useful
entry is: ID/date, prior assumption, observed evidence, decision and alternatives,
user-visible effect, verification, and remaining action. Preserve why a choice
changed even while updating the current plan. Do not log every command or fixed
compiler error. A task with no material deviations can say so.

Suggested prompt:

> Record material departures from the accepted task or plan as they arise.
> Explain the original assumption, observation, choice, alternatives, observable
> effect, and verification or next action. Continue with reversible choices in
> scope. Preserve consequential unresolved decisions explicitly. Reconcile
> durable shortcomings into the existing gap register without duplicating it.

### After implementation

Summarize before/after behavior, the execution path, consequential decisions,
failure/recovery semantics, checked evidence and remaining uncertainty. The
existing PR summary can be enough for a small task. Reuse the plan for larger
work. A quiz is optional and should ask operational questions, not recall trivia.

Suggested prompt:

> Produce a reviewer brief from the diff, plan, decisions, acceptance and actual
> verification report. Explain what changed, why, how to observe it and what can
> still fail. Link material claims to code, tests, observed driver results or
> labeled assumptions. Distinguish simulations from execution and completion
> from release readiness. When useful, add scenario questions with explanatory
> answers and evidence references. Do not infer approval from quiz completion.

For a future persistence change in this starter, the preflight could identify
uncertainty about the save commit point; implementation notes could explain why
an error after rename cannot promise rollback; the brief could show the before-
and after-rename outcomes. A comprehension question could ask what a caller can
safely conclude after each error. Existing tests provide relevant evidence, but
this example is a proposed workflow, not a new test or implemented feature.

## Implementation sequence

1. **Pilot using existing records.** Update the plan template with compact
   preflight/deviation/reader-summary guidance; update completion guidance and
   the PR template to link to that one record. Keep the four current review
   questions. Add a short repository prompt menu, register new Markdown in
   harness/docs.json, and link it from the root agent map. No Rust changes,
   network calls, new model dependencies or blocking interview are needed.
2. **Evaluate on representative tasks.** Include a small fix, an unfamiliar
   cross-boundary feature, and a persistence/compatibility change. Ask a reviewer
   to explain behavior, failure semantics and unverified boundaries using the
   artifacts. Record authoring/review effort, factual corrections, duplicated
   content and whether a consequential misunderstanding was caught. Do not use
   agent-generated quiz scores as evidence of effectiveness. Select success
   thresholds with maintainers before deciding on enforcement.
3. **Only then add structured support if warranted.** Consider an explicit
   versioned task-schema extension with a knowledge-record reference and stage
   dispositions such as recorded or not-applicable with rationale. Keep rich
   prose in the plan/brief. Preserve archived v1/v2 records and define the active
   migration policy. Add validation in task_record.rs and completion.rs, with
   positive and negative integration tests. A structural gate can validate
   references and dispositions; it cannot certify that uncertainty was found.
4. **Add optional presentation after the records prove useful.** A proposed
   command such as `cargo xtask task-view --task ID --report PATH` could render
   escaped, self-contained HTML from the authoritative records. This command
   does not exist today. Include source identity, task ID, generation command,
   evidence links and proposal/simulation/observed labels. Never require HTML
   generation for routine builds or equate its existence with readiness.

Keep durable reasoning in Git, near the task or feature. Generated HTML and
verification reports may live under target/task-artifacts/ID, with CI retention
configured separately if adopters want shared review links. A task record should
link to durable source, not only to an ephemeral target file. Finalize source
records before CI. Put the final source fingerprint in generated reports rather
than embedding a document's own full-workspace hash inside itself.

Git history preserves revisions but a final rewritten plan does not establish
when the preflight actually happened. If sequence provenance matters, retain
milestone commits or externally recorded CI artifacts. Neither is supplied by
an attractive HTML page alone. Stable product/architecture docs explain current
behavior; archived task artifacts explain a particular change at its revision.

## Acceptance for a future implementation

Documentation pilot: a reviewer can find one authoritative explanation for each
pilot task, distinguish expectation from observation, trace material decisions,
and identify remaining actions without needing the original chat. Routine tasks
can remain brief. Existing independent behavior checks remain unchanged.

If structured validation is added, test valid minimal tasks, justified skipped
optional stages, missing records, unknown fields/versions, invalid/outside-root
references, retained historical records, partial/blocked handoffs, stale CI after
source edits, and missing required acceptance. Do not add a test that merely
asserts a generated explanation agrees with the same generator.

If rendering is added, verify escaping of task/code content, absent evidence,
source identity mismatch, and simulations that cannot be displayed as verified
execution. A quiz outcome must never alter complete/release results. Completion
still requires current full-CI evidence and actual assigned requirements.

## Limits and reviewer cautions

These artifacts are generated explanations, not independent correctness evidence.
The reference-port example illustrates why: its displayed reset-to-current-time
logic at eight tokens/second with 20 ms polling implies 140 ms between mints,
although its prose suggests roughly 125 ms. This is a review inference from the
shown algorithm, not a measured execution of that example.

The implementation-notes example chooses a storage-retention policy while treating
human confirmation as nonblocking. A choice described as conservative still needs
appropriate authority when it changes destructive or externally visible behavior.
The pitch should become a balanced brief rather than an instruction to persuade.
The quiz example uses embedded answers and shipping language; passing such a quiz
establishes neither correctness, reviewer identity nor approval.

Avoid eleven compulsory documents, invented unknowns, polished but unsupported
claims, duplicated specifications, and stale descriptions presented as current
product behavior. Human comprehension and artifact usefulness need a pilot.

## Review evidence and handoff

Inspected product/architecture contracts, task schema and completion code, plan
and PR templates, readiness requirements, and ranked gaps. Doctor passed. Baseline
fast and CI runs both failed before edits in two completion tests during temporary
Git repository initialization, with Git reporting an existing template destination.
Other baseline CI checks passed: formatting, architecture, docs, coordination,
Clippy, rustdoc, scenarios and workflows. Full CI was therefore not green.

Reproducer: `cargo xtask check fast` or `cargo xtask check ci` on the inspected
base in this macOS environment. The fixture uses system time and process ID for
names with non-exclusive directory creation in xtask/tests/support/mod.rs.
Concurrent fixture-name collision is a plausible cause, not confirmed root cause.
Follow up with exclusive unique allocation and a concurrent isolation regression
if investigation confirms it. This review does not repair that unrelated failure.

Baseline evidence is under target/harness/runs/1793201960117912000-57739/report.json
and target/harness/runs/1793202001959571000-66595/report.json. Final documentation
verification and handoff reports are emitted under target/verification/unknowns-review/.
The task remains a formal partial handoff: no existing registered requirement
represents this advisory assessment, and no implemented workflow is claimed.
