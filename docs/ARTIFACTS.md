# Task pages

A task leaves HTML pages that let a human understand it: what was unknown
before code and why the result is ready to merge. The method is Thariq
Shihipar's [Know your unknowns](../examples/thariq/index.html): each page is
what a capable model produces from **one short, specific prompt, given its full
attention**. Rules and checklists do not make these pages good. The prompt, a
strong example, a dedicated author and an independent reviewer do.

Pages cost more than the code they describe, so a task writes as few as its
risk calls for. `cargo xtask task-artifact next --task ID` names the next step
for whichever tier the record declares.

## Task size

A task record carries `"tier": "small"`, `"standard"` or `"full"`; a record
without a tier is standard. Choose the tier at task start: pages due before
code must be committed before the first product-code commit, so a task cannot
move up a tier after its code lands.

| Tier | Pages | Choose it when |
| --- | --- | --- |
| small | none | At most 100 changed lines, away from policy, dependency and ceremony paths. |
| standard | `brief.html` before code, `report.html` after | Everything else. This is the default. |
| full | the eight-page set below | The change is risky or hard to undo: a persistence or wire format, a security boundary, a migration, a public contract, or a design a person has asked to steer. |

Every tier owes its task record, the four review answers, current CI evidence
and any transition receipt. The gates never force a task to be full; the owner
or the ticket asks for it.

A small task needs no pages and no page reviews. The completion and handoff
gates measure the branch against its merge base and refuse a small task when:

- its changed lines, added plus deleted, exceed 100 outside administrative paths
  (moves count on both sides; untracked files count; any binary file refuses);
- it touches a protected policy path, any `Cargo.toml`, or the ceremony's own
  definitions (this guide, the author-artifact, review-artifact and write-ticket
  skills and their `docs/skills/` pointers, and the agent folders under
  `.claude/` and `.codex/`).

Any other skill can change in a small task, with no transition receipt.
`cargo xtask docs` holds it to Markdown with only a name and description, and
no `!` commands (DOC011 to DOC013); agent host folders exist only at the
repository root (DOC014). The list is `CEREMONY_SKILLS` in
`xtask/src/transitions.rs`; a skill that needs hooks, tool grants or scripts
joins it.

Every skill keeps its `.claude/skills/` and `.agents/skills/` copies identical
(DOC007). The investigate skill is also an experiment's seed: its body must
equal `docs/skills/investigate.md` (DOC009, `SEEDS` in `xtask/src/checks.rs`),
so a change to it edits all three files, and only a measured, independently
reviewed improvement should replace the seed (README.md).

A small task that outgrows the tier is split, or restarted as a standard task on
a new branch. The limit counts lines, not meaning: dense or minified lines count
once, and one feature split across several small branches passes each gate. CI
and reviewers remain the check on both. The limits live in `xtask/src/size.rs`;
changing them needs a transition receipt.

Records written at schema version 6 or earlier called the eight-page set
"standard" and made it the default. They keep that meaning and stay valid; a
new record uses schema version 7.

## The standard pages

Two pages, one on each side of the code:

| When | Page | File | Prompt | Exemplar |
| --- | --- | --- | --- | --- |
| Before code | The brief | brief.html | "I'm about to {task} in {area}. Before I write code, give me one page: the unknowns in this part of the codebase that the ticket does not show, each with the code that proves it, then the plan, leading with the decisions I'm most likely to tweak and burying the mechanical work at the bottom." | [08](../examples/thariq/08-implementation-plan.html) |
| Before merge | The report | report.html | "I built {task}. Give me one page I can decide the merge from: lead with the demo and the evidence, then what the code forced to change from the brief, logged as deviations with the conservative option taken, then what is still open." | [09](../examples/thariq/09-implementation-notes.html) |

The exemplars were written for the full set, one job per page. A standard page
does two jobs in less room:

> A standard page is short. Aim for about half its exemplar's length: at most
> five unknowns or deviations, each proven by code or captured output, and one
> decision per screen. Cut anything a reader would learn as fast from the diff.

`next` hands this rule to the author and the reviewer of a standard page.

## The full set

Eight pages, for a change that needs a person to steer it before code and to
understand it before merge:

| When | Page | File | Prompt | Exemplar |
| --- | --- | --- | --- | --- |
| Before code | Blindspot pass | blindspots.html | "I'm about to {task} in {area}. Do a blindspot pass: find my unknown unknowns in this part of the codebase, explain each one with the code that proves it, and tell me how to prompt you better for the implementation." | [01](../examples/thariq/01-blindspot-pass.html) |
| Before code | Brainstorm the intervention | interventions.html | "Here's the problem: {problem}. Search the codebase and brainstorm the places we could intervene, from cheapest to most ambitious. I'll tell you which ones resonate." | [05](../examples/thariq/05-churn-brainstorm.html) |
| Before code | The interview | interview.html | "Interview me one question at a time about anything still ambiguous in {task}. Prioritize questions where my answer would change the architecture." | [06](../examples/thariq/06-interview.html) |
| Before code | Point at a reference | reference.html | "{reference} already behaves the way {task} should. Read it and show me a semantics map, with matched excerpts, gotchas and edge cases, so I can confirm you understood it before you build anything." | [07](../examples/thariq/07-reference-port.html) |
| Before code | The tweakable plan | plan.html | "Write an implementation plan for {task} as HTML, but lead with the decisions I'm most likely to tweak: data model changes, new type interfaces and anything user-facing. Bury the mechanical work at the bottom; I trust you on that part." | [08](../examples/thariq/08-implementation-plan.html) |
| During | Implementation notes | implementation.html | "Keep implementation notes as you build {task}. When the code forces a deviation from the plan, pick the conservative option, log it under Deviations, and keep going." | [09](../examples/thariq/09-implementation-notes.html) |
| Before merge | Buy-in doc | buy-in.html | "Package the plan, the implementation notes and the evidence into a single doc I can share to get buy-in on shipping {task}. Lead with the demo." | [10](../examples/thariq/10-pitch-doc.html) |
| Before merge | Quiz me before I merge | quiz.html | "I want to make sure I understand everything that happened in this change before I merge. Give me an HTML report on the diff, covering context, intuition and what was done, with a quiz at the bottom that I must pass." | [11](../examples/thariq/11-change-quiz.html) |

When no external reference exists, the reference is the existing code path that
the change extends or most resembles. Add these when the task calls for them:
[teach me my unknowns](../examples/thariq/02-color-grading-explainer.html)
for unfamiliar domains, [design directions](../examples/thariq/03-design-directions.html)
for visual work, and [mock before you wire](../examples/thariq/04-toolbar-mock.html)
for new UI.

Write implementation notes during the work: after the first implementation slice
and at each material milestone. Never write them all at the end.

## Pages need a reader

The interview asks a person questions and the quiz tests a person. A record
says who will read its pages: `"reader": "human"` when someone has agreed to
answer before code and to take the quiz before merge, `"reader": "none"`
otherwise. A record that names no reader has none.

Without a reader a full task writes six pages: no `interview.html` and no
`quiz.html`. Their content is not dropped. `next` hands this rule to the
authors of the plan, the buy-in doc and both standard pages:

> No person will answer this task's pages. Put each question you would have
> asked into this page with your recommendation, label it as the agent's call,
> and list the calls still open where the reader decides on the merge.

Set the reader honestly at task start. A page that waits for an answer nobody
gives costs a full author and review loop and tells the next reader nothing.

## How to produce a page

`cargo xtask task-artifact next --task ID` reads the task record, the pages and
the task's verdict log and prints the one next step: author, review, rewrite,
commit, build or complete. Author, review, rewrite and build steps carry a
`handover`: the page's prompt from the tables above with its braces filled from
the record's `prompt` values, the exemplar, the output path, the task context
and the pages so far, plus the last review's findings for a rewrite. Pass it to
the subagent unchanged; `unfilled` names any `prompt` value the record lacks,
whose brace stays in the text. It advises and never blocks. It exits 3 when it also
has a warning, such as a page edited after its last review. Completion does not
read the log.

1. **Use a dedicated author.** Start a fresh subagent for each page and give it
   the [author-artifact brief](../.claude/skills/author-artifact/SKILL.md) and
   the step's `handover`: the prompt with the task filled in, the exemplar
   path, the task context and the task's pages written so far. If your host has no subagents, give the page its own turn and
   do nothing else in it. Write the pages in the order of their table.
   Authors working in parallel each derive the same headline finding again; one
   after another, each builds on the last.
2. **Read the exemplar first.** The author reads the matching page in
   [examples/thariq/](../examples/thariq/) in full before writing. Match its
   depth, density and the job its interactions do, not its layout or domain.
   Those pages run 27–41 KB. A full-set page a fraction of that size on a
   comparable task usually summarizes where it should show; a standard page
   follows its own length rule above.
3. **Ground every claim.** Name the file, function, test or commit behind each
   finding. Do not invent findings, options, questions or reference code to fill
   space. A small change gets a smaller page, but it must still teach something
   the diff alone does not. A finding an earlier page showed gets a one-line
   citation in later pages, not a second derivation.
4. **Look at it.** Run `cargo xtask task-artifact screenshot --task ID`. It
   renders each page with local Chrome at desktop and phone width into
   `target/artifact-screenshots/ID/` and reports console errors, phone-width
   overflow, network requests and any file the page loads from disk. View the
   tiles next to the exemplar's
   (`node scripts/artifact-screenshots.cjs OUT_DIR examples/thariq/PAGE.html`)
   and fix what the report flags before committing. Without Node or Chrome, say
   so rather than implying visual review.
5. **Get an independent review.** Start another fresh subagent, never the
   author, with the [reviewer brief](../.claude/skills/review-artifact/SKILL.md) and
   the review step's `handover`: the page, its prompt, the exemplar and the task
   context. A different model is better when
   your host offers one. On REVISE, give the findings to a fresh author to
   rewrite the page, then have a fresh reviewer judge the whole page again:
   each review samples different claims, and rewrites can break what was
   right. After two rewrites, commit the best
   version. Record every review with `cargo xtask task-artifact verdict --task ID
   --page PAGE --verdict pass|revise|best --reviewer NAME --model MODEL
   --findings N --notes -`, piping the reviewer's whole reply to stdin, where
   shell quoting cannot cut it (at most 64 KiB; `--notes TEXT` takes it inline).
   N counts every finding the reviewer listed, blocking or not, and MODEL is the
   reviewer's model as the host names it. The rewrite step hands those notes to
   the next author. It
   appends a row with the page's SHA-256 to `reviews.json` in the task's folder
   and keeps a copy of the reviewed version under `target/artifact-reviews/`;
   `best` marks the reviewed version restored after two rewrites. Commit the log
   with the page; the artifact author's hook refuses edits to it. Copy each
   page's final verdict and any findings left open into the task record's
   `review.verification`.

Fill in the braces from the task: `next` fills them from the record's `prompt`
values (`task` in the two prompts that open "I'm about to", `task_noun` in the
other `{task}`s, `area`, `problem`, `reference`) and reads the tables by their
File column. The adapted prompt is the author's instruction. The exemplar is
the bar.

## Replies come back into the work

Thariq's pages end with a reply the reader assembles and copies back. When the
record's reader is human, present the pages due before code and wait for the
reply before writing product code; the answers are task input. If the reader
does not answer after all, follow this rule, which `next` hands the interview
and buy-in authors:

> When no one answers the discovery pages, answer each interview question with
> your recommendation, label it as the agent's call, and carry the questions
> still open into the buy-in doc.

## Keep pages about the feature

A page speaks to a reviewer about the change. Keep harness mechanics out of it:
commit hashes, revision numbers, verification boilerplate and paragraphs
about what the page is not. One short label is enough when a page shows a model
rather than the running product. Report verification limits to the caller and in
the task record.

## Files and commits

The harness checks presence, a floor of text and order. Quality is the
reviewer's and the human's call.

1. After creating the task record, run `cargo xtask task-artifact style --task ID`.
   It creates `harness/task-artifacts/ID/` and writes the shared stylesheet to
   `target/artifact-style.html` as a `<style data-artifact-foundation="1">`
   block. Paste that block first in each page's head and use its tokens for
   color, type and spacing. Layout stays page-specific.
2. Write the pages in that folder under the file names in the tables, with the
   verdict log `reviews.json`. Each page is one self-contained file that loads
   nothing else, not even from its own folder, so it renders the same in a file
   preview, on a code host or as an attachment, and a later style change never
   alters it. A required page holds at least 1,200 characters outside
   its tags and styles; a page that is only the stylesheet is refused.
3. Commit as you go. Completion and handoff read the branch's own commits since
   the base. At any stage, the pages due before code come before the first
   product-code commit. At completion, a full task's notes come before the last
   product-code commit, and the pages due before merge (the report, or the
   buy-in doc and quiz) at or after it. Code and pages must be committed when
   you claim completion. Git history keeps every earlier version.

On a pull request a code host checks out a merge commit it built, from which
the branch's own commits cannot be read. CI passes the branch tip to
`cargo xtask handoff --base REV --head REV` so the order is checked there too.

`cargo xtask task-artifact guard --task ID` fails until the pages due before
code are committed. The opt-in [Codex example](../examples/codex-artifact-hooks.json) runs it
before `apply_patch` edits to product code. It does not cover shell or MCP
writes. The default stop hook gates completion claims; see
[COMPLETION.md](COMPLETION.md).

Examples are from Thariq Shihipar's
[html-effectiveness](https://github.com/ThariqS/html-effectiveness), vendored
unmodified under Apache-2.0; see [their provenance](../examples/thariq/README.md).
