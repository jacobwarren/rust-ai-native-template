# Why this template is small

This repository once carried a large harness for agent work. It was measured
on this codebase and removed. This page records what was measured, what was
not, and how to measure an addition of your own before you keep it. The full
write-up, with the protocol and every limit, is `docs/EVALUATION.md` on the
`showcase/full-harness` branch.

## What was removed

- Task pages: up to eight self-contained HTML pages per task (blindspots,
  interventions, interview, reference, plan, notes, buy-in, quiz), each written
  by an author agent and judged by a reviewer agent, with up to two rewrites.
- A task record per change, with four review answers, and a completion gate
  that checked it against CI evidence.
- Transition receipts binding a rationale to hashes of protected files.
- A custom check runner with evidence reports, process workflows and golden
  files, in about 3,700 lines of Rust.
- Hooks for Claude Code and Codex: an edit guard, a post-edit check and a stop
  gate.

## Experiment 1: do task pages help?

Five features of the task board (`rename`, `reopen`, `clear_done`, `redo`,
`stats`), each built by the same model with no pages, with two pages (a brief
before code and a report after), and, for two of them, with six. Each run was
graded by 11 to 15 held-out checks against the built binary, plus CI.

| | No pages | Two pages | Six pages |
| --- | --- | --- | --- |
| Accepted | 5 of 5 | 5 of 5 | 2 of 2 |
| Wall-clock time per task | 3 min | 33 to 54 min | 95 and 126 min |
| HTML written per task | none | 507 to 668 lines | 2,585 and 2,629 lines |

Pages bought no accepted change. Most review rounds were about the page's
length or a citation, not the code.

## Experiment 2: does the rest of the harness help?

Five harder tasks (reorder with `move`, raise the limits, an atomic
`add_many`, a new `already_done` error, a `find` method), each built twice in
the harness with pages already removed, and twice in a stripped copy that is
close to what this repository is now. The prompt was only the request. Each
run was graded by 13 to 15 held-out checks.

| | Stripped copy | Harness |
| --- | --- | --- |
| Accepted | 8 of 10 | 7 of 9 |
| Held-out checks passed | 140 of 142 | 125 of 127 |
| Mean wall-clock time | 94 s | 252 s |
| New tests per run | 1.6 | 2.4 |
| Updated the docs | 10 of 10 | 9 of 9 |
| Tests, Clippy and formatting clean | 10 of 10 | 9 of 9 |

- Every failure in both arms was the same check: after raising the journal
  limit, a full journal no longer fit the store's byte cap and could not be
  saved. The harness's own CI passed on the runs that failed.
- Stripped runs added tests and updated the docs without being asked.
- Eight of nine harness runs skipped the task record as out of proportion.
  The one that followed the process left a product doc un-updated to avoid
  needing a receipt.

## Experiment 3: does a post-edit hook help?

The hooks were the one removed part the first two experiments could not
reach, so a small one was rebuilt and tested with hooks firing: 24 lines that
type-check the edited crate and check its formatting after every edit, and
hand any failure back to the agent. It is on the `harness/post-edit-hook`
branch. The runs were headless sessions in fresh clones of this repository,
with and without the hook, on the five tasks of experiment 2, three runs per
cell, on Claude Sonnet 5.5 and on Claude Haiku 5.5 as a weaker model.

| | Sonnet, no hook | Sonnet, hook | Haiku, no hook | Haiku, hook |
| --- | --- | --- | --- | --- |
| Accepted | 15 of 15 | 15 of 15 | 15 of 15 | 15 of 15 |
| Held-out checks passed | 213 of 213 | 213 of 213 | 213 of 213 | 213 of 213 |
| `scripts/ci.sh` checks clean | 15 of 15 | 15 of 15 | 15 of 15 | 15 of 15 |
| Mean wall-clock time | 96 s | 94 s | 143 s | 149 s |
| Mean cost | $0.32 | $0.31 | $0.07 | $0.08 |
| Mean turns | 24.9 | 24.7 | 36.8 | 38.1 |
| Times the hook ran | 0 | 115 | 0 | 179 |

The hook ran 294 times and changed nothing that was measured: not what was
accepted, not whether the work was clean, not the time, cost or number of
turns. It was not added to this template.

Two things qualify that. Every run passed everything, so the set had no room
to show a benefit; a hook like this may matter on a codebase where a build
takes minutes and an agent would otherwise go a long way on broken code. And
the runs did not record how often the hook reported a failure rather than
passing silently, so "it never needed to speak" and "it spoke and made no
difference" cannot be told apart.

One result is new. The `limits` task failed in all four runs of experiment 2
and passed in all twelve here. The difference is the checklist in
[AGENTS.md](../AGENTS.md), which did not exist then and names that exact
trap. That is the checklist being written from the test, not evidence that
checklists generalize; but it is the only change in any of these experiments
that moved an acceptance number, and it is six lines of text.

## A held-out build

One agent was given the harness, with no history, and a specification for a
different product: a small HTTP link shortener. It replaced the example in 57
minutes and passed 19 of 20 held-out checks made over real HTTP; the miss was
an ambiguity in the specification. To grade its own service it first had to
rewrite the harness's workflow runner, which assumed a program that exits.
There was no stripped build to compare it with.

## What this does not show

- **Only one hook was tested.** Experiment 3 covers a post-edit check. The
  edit guard and the stop gate of the removed harness were never run with
  hooks firing.
- **Nothing here spans sessions or people.** Records and receipts exist to
  coordinate work over time. Single-task runs cannot show whether they do.
- **Little headroom.** In experiment 1 every no-pages run passed, in
  experiment 2 four of five tasks were passed by every run, and in experiment
  3 every run passed everything, including on the weaker model. These tasks on
  this codebase are too easy to show what helps on hard ones.
- **Small samples.** One to three runs per cell, two models, one small
  codebase. In experiment 3 the headless agents had a narrow shell allowlist
  and were refused three to five commands per run, in both arms alike. Time was measured; tokens were not, beyond the top-level
  agent.
- **The evidence is not in this repository.** Prompts, graders and run
  clones lived in a session scratch directory. The tables are a record, not
  something you can re-derive from this tree.

Briefs written before changes to the harness's own gates did find real
defects in three of four cases. That is anecdote from four changes, and the
gates they protected are gone.

## Before you add process

A stronger-looking setup does not show that agents do better. Measure it.

1. **Build tasks from real need**, each with a starting commit, a prompt as a
   user would write it, and a grader the agent cannot see that drives the real
   product. Check that the grader fails on the starting commit.
2. **Check the set has headroom.** If the plain setup passes everything, the
   set cannot show an improvement. Add harder or less specified tasks until
   some runs fail.
3. **Run pairs.** Same model, same prompt, same starting commit, with and
   without the one thing you are testing, in separate clones, more than once.
4. **Grade independently.** The agent's own "done" is not the grade. Record
   time and cost, and the failures in both arms.
5. **Group failures by cause before changing anything.** A wrong grader or an
   ambiguous task is fixed in the task, not in the prompt.
6. **Keep the addition only if it changes the result** by more than its cost,
   and write down what the run could not test.

The lesson these runs did yield is in [AGENTS.md](../AGENTS.md): a short
checklist of what agents left out when the request did not spell it out. It
came from grading and costs a few lines to read. Its one test so far is the
`limits` result above, on the task it was written from.
