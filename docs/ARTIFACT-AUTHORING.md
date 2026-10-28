# Author artifacts that help people reason

Read this before writing task HTML. The required deliverable is a useful human
review experience, not eight files that happen to pass registration. The shared
foundation supplies visual consistency; you supply the composition, evidence,
explanations and working interactions. No fixed layout, DOM schema, word count,
card count, or JavaScript framework is required.

## Study techniques, then design for the task

Inspect the [worked persistence studies](../examples/artifact-studies/index.html)
and [Thariq Shihipar's gallery](https://thariqs.github.io/html-effectiveness/unknowns/index.html).
The local examples deliberately use different compositions: an evidence dossier,
an intervention comparison, a branching interview, a semantic map, a live failure
model, a deviation timeline, a demonstration with objections, and a scenario quiz.
They are examples to learn from, not templates to fill in or screenshots to clone.
Their persistence subject is existing repository behavior, not a requirement to
add persistence to every feature. Sample proposals are labeled as proposals.

Before writing HTML, identify what the reader needs to understand or decide, the
non-obvious behavior, the source evidence, and the representation that makes it
clear. A state transition may need a stepper; a schema choice may need annotated
types and changed payloads; visual design may need side-by-side mocks. A small
change can use a compact explanation. It still needs the relevant reasoning.

Prefer explaining one real tradeoff deeply to filling a long page with invented
options. Use progressive disclosure to keep mechanical detail available without
burying the consequential choice. Give each page a clear title, orientation,
readable hierarchy, navigation to related explanations and a concrete response
mechanism. Use shared tokens with custom geometry. Do not mistake whitespace,
decorative cards, unrelated animation or repeated prose for explanatory richness.

## Outcomes for each purpose

| Purpose | What the reader should be able to do | Useful techniques; choose or invent your own |
| --- | --- | --- |
| Blindspots | Trace a non-obvious risk from observed code to a failure scenario and an actionable constraint | Evidence excerpts, risk-to-boundary diagram, selectable prompt corrections. State inspected scope and uncertainty. |
| Interventions | Compare credible approaches and understand why one is recommended | Effort/benefit map, side-by-side behavioral examples, tradeoff table, selections that compose a proposed scope. Include the cost of doing nothing when relevant. |
| Interview | Resolve consequential ambiguity without guessing what changes downstream | Order by architectural impact; show consequences of each answer, allow a custom answer, retain a reviewable decision summary. Distinguish user answers, agent recommendations and delegated choices. Settled questions may be shown with their actual basis. |
| Reference | Verify that the agent understood the relevant semantics, not just the names | Matched real excerpts or contracts, annotated differences, inputs with independently expected outputs, linked highlights. Cite exact sources; label pseudocode and proposed target behavior. A link alone is not a semantic map. |
| Tweakable plan | Explore a choice and see how the implementation and its acceptance change | Annotated interfaces, state/history simulation, changed payloads, execution dependencies, expected tests. Put likely-to-change choices first and mechanical steps behind disclosure. |
| Implementation notes | Understand what the code taught the agent while work was in progress | Plan → observation → decision → consequence entries with source/test evidence and remaining questions. Make deviations and their impact scannable. A command transcript alone is insufficient; do not invent a deviation when work followed the plan. |
| Buy-in | Explain the feature to another person and evaluate objections | Before/after walkthrough or labeled model, concrete user scenario, what/why/where, evidence-linked objections, limitations and review responsibilities. A model is not a running product; label the boundary. |
| Quiz | Predict consequential behavior and repair a mistaken mental model | Scenario inputs, answer commitment, specific feedback for plausible wrong answers, links to the exact explanatory sections, retry or response export. A reveal-only answer list is reference material, not a comprehension check. Scores never grant approval. |

The reader outcomes in the middle column are required, not aspirations. The
techniques in the last column are optional ways to achieve them. Choose another
representation when it serves the task better; do not omit the underlying outcome.
In particular:

- The buy-in must walk through a substantive feature scenario, showing relevant
  starting state, user action, resulting state and a consequential boundary or
  failure. A static annotated sequence, interactive model or actual demonstration
  can work. A command sample plus a summary table alone does not provide this
  walkthrough. Label modeled behavior separately from observed execution.
- Evidence must be traceable: link the actual source/test/report and show the
  pertinent excerpt or independently expected scenario. Plain-text paths and a
  generic “tests passed” paragraph do not let a reviewer follow the claim. Reports
  under target may be ephemeral; label that limit and retain stable test links.
- Quiz scenarios must elicit a prediction before showing feedback. Explain the
  specific mistaken model and link to the relevant explanatory section; permit
  retry. Reveal-only questions may supplement the quiz but do not satisfy it.
  Treat an empty/cleared answer as unanswered and remove stale feedback/exports.
- Every page needs its own accurate document title and heading. Navigation must
  lead to the artifacts available in that revision, including notes and final
  pages once present; no copied discovery-only navigation on final documents.

These are content and usability outcomes, not a mandated layout or element count.
Additional explainers, design directions or throwaway mocks remain optional when
they reduce uncertainty.

## Consequential interactions

A control must help the reader explore, compare, predict or respond. For example,
choosing a save-failure boundary can update the visible committed state, the
returned error, the recovery action and the expected regression. A dropdown that
merely repeats the option in a sentence proves little. Changing two strings in a
test is not evidence that a decision tool is useful.

Keep displayed consequences and exported responses synchronized. Export the
selected scenario, relevant assumptions and proposed decision in readable text
or JSON. Defaults are recommendations, not human decisions. Provide a usable
fallback when clipboard/download is unavailable, such as a labeled selectable
response. Page-local controls do not modify the product or confer approval.

Test actual handlers with independent expected results. Assert the meaningful
state, explanation or acceptance change and the response contents. For a plan,
exercise at least two alternatives, including a round trip back to a prior
choice. Test additional interactions that carry important reasoning. Use test
data distinct from page implementation wherever practical. Record DOM tests as
DOM tests; they cannot establish browser appearance or native clipboard behavior.

## Evidence and review

Link claims to actual source, tests, contracts or reports. Show the relevant
excerpt or expected scenario on the page so the reader can understand it without
opening every link. Distinguish observed results, proposed behavior and assumptions.
Include source identity or revision when comparison depends on it. Avoid dangling
“tests passed” claims and unrelated evidence links.

### Independent review loop for nontrivial feature tasks

Use a separate review subagent when that capability is available. This repository
instruction authorizes that bounded delegation. The author still owns repairs
and registration; the reviewer does not silently rewrite artifacts. If independent
review is unavailable, perform the same self-review and explicitly record that
limitation. Do not describe self-review as independent verification.

1. The author reads this guide and inspects the relevant local worked studies
   before composing. Draft the five discovery artifacts and test their important
   interactions. Before product implementation, give the reviewer the task intent,
   actual pages, inspected source, this guide and the relevant reference examples.
2. Ask the reviewer to assess each required reader outcome, evidence traceability,
   consequential choices, response consistency, titles and navigation. It must
   compare how the artifacts support reasoning with the reference techniques,
   not merely count files or praise the formatting. Source/DOM review is useful
   when rendered access is unavailable, but cannot establish appearance.
3. Record concrete findings and repair substantive misses before product code.
   Have the reviewer recheck repaired outcomes. Register the reviewable discovery
   bundle, keeping the findings and disposition as supporting evidence. An honest
   unresolved blocking product question still requires clarification.
4. Keep notes at the required implementation checkpoints. Before final
   registration, repeat independent review on the complete bundle, especially the
   feature walkthrough, objection/evidence chain and quiz feedback/remediation.
   Fix issues and recheck them before registering final artifacts. If repairs
   change product code, follow the normal implementation checkpoint sequence.

Review notes may be freely authored HTML or a supporting report: identify the
reviewed revision/content, findings, repairs, unresolved limitations and checks
actually performed. No fixed report schema is required. A reviewer saying “looks
good” without inspecting the pages and their source basis is not this review.
The loop improves reliability; it cannot guarantee judgment or replace rendered
human review, artifact integrity, product tests or merge authorization.

Review the finished bundle against these questions before handoff:

1. Can a reader explain the non-obvious behavior and why it was chosen, using
   the artifact alone? Is there enough evidence to challenge the explanation?
2. Does each interaction change something meaningful, and does its response
   accurately preserve the choice? Are models honest about what they simulate?
3. Does each page use a representation suited to its subject, with hierarchy,
   readable line lengths and enough context to orient a new reviewer?
4. Were the actual rendered pages inspected at wide and narrow widths, with
   expanded content, keyboard focus and long text? Record screenshots and exact
   interactions reviewed. If access is blocked, say visual review is pending.
5. Compared with the relevant reference technique, what reasoning does this
   artifact enable, and what remains missing? Do not require matching aesthetics.

Report lifecycle integrity, semantic/interaction testing, and visual review as
separate results. A green registry or CI report cannot stand in for the latter two.
Human utility remains a qualitative judgment; do not add keyword or element-count
checks and call them a quality guarantee. If the artifact is shallow, improve the
content and design before declaring success, even if every file check passes.

For fresh-agent trials, seed the guidance and examples before dispatch. Give an
ordinary feature request without artifact reminders. Observe when artifacts are
created, inspect their meaning and compare rendered output with the references.
Post-hoc repairs by the evaluator must be disclosed and do not count as uncoached
generation success. Preserve failed runs as useful evidence.
