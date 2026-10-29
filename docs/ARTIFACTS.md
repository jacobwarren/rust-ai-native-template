# Task artifacts

Every task leaves HTML pages that let a human understand it: what was unknown
before code, what the code taught us during the work, and why the result is
ready to merge. The method is Thariq Shihipar's
[Know your unknowns](../examples/thariq/index.html): each page is what a capable
model produces from **one short, specific prompt, given its full attention**.
Rules and checklists do not make these pages good. The prompt, a strong example,
a dedicated author and an independent reviewer do.

## How to produce a page

1. **Use a dedicated author.** Start a fresh subagent for each page and give it
   the [author-artifact brief](skills/author-artifact.md), the prompt below with
   the task filled in, the exemplar path and the task context. If your host has
   no subagents, give the page its own turn and do nothing else in it.
2. **Read the exemplar first.** The author reads the matching page in
   [examples/thariq/](../examples/thariq/) in full before writing. Match its
   depth, density and the job its interactions do, not its layout or domain.
   Those pages run 27–41 KB. A page a fraction of that size on a comparable task
   usually summarizes where it should show.
3. **Ground every claim.** Name the file, function, test or commit behind each
   finding. Do not invent findings, options, questions or reference code to fill
   space. A small task gets a smaller page, but it must still teach something
   the diff alone does not.
4. **Look at it.** Run `cargo xtask task-artifact screenshot --task ID`. It
   renders each page with local Chrome at desktop and phone width into
   `target/artifact-screenshots/ID/` and reports console errors, phone-width
   overflow, network requests and any file the page loads from disk. View the tiles next to the exemplar's
   (`node scripts/artifact-screenshots.cjs OUT_DIR examples/thariq/PAGE.html`)
   and fix what the report flags before committing. Without Node or Chrome, say
   so rather than implying visual review.
5. **Get an independent review.** Start another fresh subagent, never the
   author, with the [reviewer brief](skills/review-artifact.md), the page, its
   prompt, the exemplar and the task context. A different model is better when
   your host offers one. On REVISE, give the findings to a fresh author to
   rewrite the page, then have a fresh reviewer judge the whole page again:
   each review samples different claims, and rewrites can break what was
   right. After two rewrites, commit the best
   version. Record each page's final verdict and any findings left open in the
   task record's `review.verification`.

## The prompts

Fill in the braces from the task. The adapted prompt is the author's
instruction. The exemplar is the bar.

| When | Page | Prompt | Exemplar |
| --- | --- | --- | --- |
| Before code | Blindspot pass | "I'm about to {task} in {area}. Do a blindspot pass: find my unknown unknowns in this part of the codebase, explain each one with the code that proves it, and tell me how to prompt you better for the implementation." | [01](../examples/thariq/01-blindspot-pass.html) |
| Before code | Brainstorm the intervention | "Here's the problem: {problem}. Search the codebase and brainstorm the places we could intervene, from cheapest to most ambitious. I'll tell you which ones resonate." | [05](../examples/thariq/05-churn-brainstorm.html) |
| Before code | The interview | "Interview me one question at a time about anything still ambiguous in {task}. Prioritize questions where my answer would change the architecture." | [06](../examples/thariq/06-interview.html) |
| Before code | Point at a reference | "{reference} already behaves the way {task} should. Read it and show me a semantics map, with matched excerpts, gotchas and edge cases, so I can confirm you understood it before you build anything." | [07](../examples/thariq/07-reference-port.html) |
| Before code | The tweakable plan | "Write an implementation plan for {task} as HTML, but lead with the decisions I'm most likely to tweak: data model changes, new type interfaces and anything user-facing. Bury the mechanical work at the bottom; I trust you on that part." | [08](../examples/thariq/08-implementation-plan.html) |
| During | Implementation notes | "Keep implementation notes as you build {task}. When the code forces a deviation from the plan, pick the conservative option, log it under Deviations, and keep going." | [09](../examples/thariq/09-implementation-notes.html) |
| Before merge | Buy-in doc | "Package the plan, the implementation notes and the evidence into a single doc I can share to get buy-in on shipping {task}. Lead with the demo." | [10](../examples/thariq/10-pitch-doc.html) |
| Before merge | Quiz me before I merge | "I want to make sure I understand everything that happened in this change before I merge. Give me an HTML report on the diff, covering context, intuition and what was done, with a quiz at the bottom that I must pass." | [11](../examples/thariq/11-change-quiz.html) |

All eight are required. When no external reference exists, the reference is
the existing code path that the change extends or most resembles. Add these
when the task calls for them: [teach me my unknowns](../examples/thariq/02-color-grading-explainer.html)
for unfamiliar domains, [design directions](../examples/thariq/03-design-directions.html)
for visual work, and [mock before you wire](../examples/thariq/04-toolbar-mock.html)
for new UI.

Write implementation notes during the work: after the first implementation slice
and at each material milestone. Never write them all at the end.

## Replies come back into the work

Thariq's pages end with a reply the reader assembles and copies back. When a
human is available, present the discovery pages and wait for their reply before
writing product code; their answers are task input. When running unattended,
answer the interview with your recommendation, label it as yours, and carry the
open questions into the buy-in doc.

## Keep pages about the feature

A page speaks to a reviewer about the change. Keep harness mechanics out of it:
commit hashes, revision numbers, verification boilerplate and paragraphs
about what the page is not. One short label is enough when a page shows a model
rather than the running product. Report verification limits to the caller and in
the task record.

## Files and commits

The harness checks presence and order. Quality is the reviewer's and the
human's call.

1. After creating the task record, run `cargo xtask task-artifact style --task ID`.
   It creates `harness/task-artifacts/ID/` and writes the shared stylesheet to
   `target/artifact-style.html` as a `<style data-artifact-foundation="1">`
   block. Paste that block first in each page's head and use its tokens for
   color, type and spacing. Layout stays page-specific.
2. Write the pages in that folder as `blindspots.html`, `interventions.html`,
   `interview.html`, `reference.html`, `plan.html`, `implementation.html`,
   `buy-in.html` and `quiz.html`. Each page is one self-contained file that loads
   nothing else, not even from its own folder, so it renders the same in a file
   preview, on a code host or as an attachment, and a later style change never
   alters it.
3. Commit as you go. Completion reads the branch's own commits since the base:
   the five discovery pages before the first product-code commit, notes before
   the last one, and buy-in and quiz at or after it. Code and pages must be
   committed when you claim completion. Git history keeps every earlier version.

`cargo xtask task-artifact guard --task ID` fails until the discovery pages are
committed. The opt-in [Codex hooks](../examples/codex-artifact-hooks.json) run it
before `apply_patch` edits to product code, and run completion at stop. They do
not cover shell or MCP writes.

Examples are from Thariq Shihipar's
[html-effectiveness](https://github.com/ThariqS/html-effectiveness), vendored
unmodified under Apache-2.0; see [their provenance](../examples/thariq/README.md).
