# Evaluate the harness, not just the code

A stronger-looking template does not establish that agents solve tasks better.
Use this protocol before adding costly semantic reviewers or model-specific tricks.
The optional executable evaluator and GEPA adapter are described in
[experiments](../experiments/README.md). The protocol below governs their use.

## Task set

Create realistic tasks from your actual codebase, with a fixed starting commit,
user prompt, observable acceptance criteria, allowed files/commands, time and
cost budgets, and independent grading checks. Keep a small development set and
separate held-out tasks that the agent cannot read. A public in-repo fixture is
not a holdout. Never expose hidden graders through agent-visible docs or tools.

Useful starter development tasks (too small for a production claim):

| Task | Independent grading idea |
| --- | --- |
| Add a greeting mode | Verify valid/invalid combinations and preserve old output |
| Repair Unicode length handling | Test ASCII, multibyte boundaries, and controls |
| Add a storage adapter | Verify layer boundaries and per-run state isolation |
| Diagnose a failed process | Require observed failure evidence, not guessed success |
| Change a response field safely | Verify consumers and documented compatibility |

## Paired comparison

Run the same model/version, prompt, tool access, resource budget, and starting
snapshot with baseline versus one harness change. Use independent worktrees and
repeat runs to expose variance. Record all failures and costs, including retries,
review, and escalation. Do not tune on held-out failures and keep calling them held out.

Record: task/version, harness commit, model/version, environment, elapsed time,
tokens/cost if known, tool calls, completion claim, independent grader result,
regressions, false-positive interruptions, human interventions, and report paths.
The coding agent's own “done” message is not the grade. Unknown cost stays null.

Set decision criteria before running: for example, more independently accepted
changes at an acceptable total time/cost, without increased regressions or false
completion claims. Set an experiment budget and stop rule. Keep or remove the
change based on the result. A handful of greeting tasks cannot demonstrate gains
across arbitrary Rust codebases.

## Check the task set before tuning against it

Before using a task set to tune a prompt, skill or harness change, check that
it can measure one. Run each check once per set, and again when tasks or
graders change:

- **Capability shows.** Scores rise with a stronger model and with more
  thinking budget. If they do not, the tasks or the grader measure something
  else, such as format or luck.
- **Headroom is left.** The best model and budget still fail some tasks. A set
  every configuration passes cannot show an improvement.
- **Runs agree.** Repeated runs of one configuration give similar scores. If
  the spread is as large as the difference you hope to see, add runs or tasks
  before comparing.
- **Tasks come from need.** Pick tasks for what the product must do, not from
  one model's failures. Failures peculiar to one model may disappear with the
  next, and a prompt tuned to them overfits.

When tuning stops improving, group the remaining failures by cause before
changing the prompt again. Ambiguous tasks and broken graders are the usual
causes, and the fix is the task or the grader, not the prompt. Record what each
group was and what changed.

## Guardrail tests

A mechanical checker should have an allowed fixture and an intentional violation.
A semantic checker also needs benign near-misses, calibration, and a way to
abstain. Prefer advisory output until measured precision supports blocking CI.
Policy edits and grader edits should receive separate review from product changes
when possible. Local scripts are mutable; enforce authority in the hosting/CI layer.

## Pages on product tasks

The paired comparison above, run once on this template's own example, is why
most tasks owe no pages ([task size](ARTIFACTS.md)).

**Set.** Five features of the task board, each specified by observable
behaviour: the commands `rename`, `reopen`, `clear_done` and `redo`, and a
read-only `stats` method. Each had a held-out grader the agents never saw: 11 to
15 checks driving the built binary over its JSON-lines protocol, covering the
stated behaviour and what `docs/DRIVER.md` promises for any new surface (strict
admission, discovery, replay, save and load, both transports). Every grader
failed on the starting commit. The graders and prompts lived outside the
repository.

**Arms.** The same model (Claude Sonnet 5.5, as subagents of one session), the
same starting commit, one fresh clone per run, one run per cell. No pages:
"implement, test, finish when `check ci` passes", with no task record. Standard:
the tier as it then stood, a task record, a brief before code and a report
after, each written by a fresh author subagent and reviewed by a fresh reviewer,
finishing at `cargo xtask complete`. Full, on two features: the six pages a
full task owes without a reader.

**Grade.** The held-out grader on a clean build of what the run committed, and
`check ci` on a clean export of it. The agent's own claim was not the grade.

| Feature | No pages | Standard (two pages) | Full (six pages) |
| --- | --- | --- | --- |
| rename | accepted, 3 min | accepted, 53 min, 5 reviews | accepted, 95 min, 10 reviews |
| reopen | accepted, 3 min | accepted, 45 min, 4 reviews | not run |
| clear_done | accepted, 3 min | accepted, 34 min, 3 reviews | not run |
| stats | accepted, 3 min | accepted, 33 min, 3 reviews | not run |
| redo | accepted, 3 min | accepted, 54 min, 6 reviews | accepted, 126 min, 12 reviews |

All twelve graded runs were accepted and passed CI. Pages bought no accepted
change and cost about 15 times the wall-clock time for two pages and 30 to 40
times for six. The standard runs wrote 507 to 668 lines of HTML each; the full
runs 2,585 and 2,629 for features of a few dozen lines. In the run reports, most
rewrites were for page length or a citation. The report page took more review
rounds than the brief in three of five standard runs, and a late code fix
forced it to be rewritten.

**What this does not show.**

- No headroom: every no-pages run passed, so this set could not have shown a
  benefit. A brief may still help on tasks a model fails without one. Build
  that set before putting pages back: tasks from real need that the best
  configuration still fails some of (see the checks above).
- One run per cell, one model, one small codebase. The time ratio is large
  enough to survive that; an acceptance difference of one run would not be.
- Wall-clock time is the cost measure. Token counts were available only for the
  top-level agent (about 96,000 without pages, 147,000 with two), not for the
  page authors and reviewers it started, so the token ratio is higher than
  those two numbers suggest and was not measured.
- The runs were subagents working in clones, so the repository's edit and stop
  hooks did not fire. Both arms lacked them equally.
- One grader check was wrong (it expected a request id on a rejected request,
  which the protocol returns as null) and was corrected before the affected run
  was counted. No other failure occurred to group by cause.

**Where a page did find something.** Three harness changes made in the same
period each wrote a brief. Two found defects before the code was committed: a
crash and a text floor that would have refused six existing pages; and an
archive check that would have failed for anyone starting without Git history.
All three changed the gates. That, and no product result, is the evidence for
keeping one brief on policy, dependency and ceremony paths.

**A held-out build.** One fresh agent, same model, was given an export of this
template with no history and a specification for a different product: a small
HTTP link shortener, std only. Following `docs/ADOPTING.md` under the standard
tier, it replaced the example product in 57 minutes, passed its own
`check ci` and completion gate, and passed 19 of 20 held-out checks made over
real HTTP (creation, redirects, listing, deletion, every error case, twenty
concurrent clients, restart, an unwritable store). The one miss was an
ambiguity in the specification: a body `{"url":5}` answered `invalid_url` where
the grader expected `invalid_request`, a call the agent recorded. It is one
build and proves little about other products. What it did show is a limit of
the harness, recorded as the gap `service-acceptance`: the workflow and
scenario runners assume a program that exits, so the agent rewrote
`xtask/src/workflows.rs` and added a service driver before it could grade its
own product.

## The harness against a stripped copy

The run above compared pages with no pages, both inside the harness. This one
asks the question underneath it: does the harness itself change what an agent
delivers?

**Arms.** *Harness*: this template after the page machinery was cut to one
brief, where a product task owes a task record and CI. *Stripped*: the same
product code and tests as a plain Cargo workspace, with a short README and the
product and driver docs, and no `xtask`, hooks, skills, agent files, `AGENTS.md`,
task records or receipts. It passes `cargo test` and clippy on its own.

**Set.** Five tasks chosen to be harder than the first set: reorder tasks with
a `move` command (the board is keyed by id, so order needs a model change);
raise the limits to 200 tasks and 1,000 commands; an atomic `add_many`; a new
`already_done` error code; a `find` method with Unicode case-insensitive
matching. Each had 13 to 15 held-out checks. The prompt was the request as a
user would write it, identical in both arms, with nothing about tests, docs or
process. Two runs per cell, Claude Sonnet 5.5 as subagents in fresh clones,
graded on a clean build of what was committed.

| Task | Stripped | Harness |
| --- | --- | --- |
| move | 2 of 2 accepted | 1 of 1 accepted |
| limits | 0 of 2 | 0 of 2 |
| add_many | 2 of 2 | 2 of 2 |
| already_done | 2 of 2 | 2 of 2 |
| find | 2 of 2 | 2 of 2 |
| Accepted | 8 of 10 | 7 of 9 |
| Held-out checks passed | 140 of 142 | 125 of 127 |
| Mean wall-clock time | 94 s (75 to 110) | 252 s (180 to 416) |
| New tests per run | 1.6 | 2.4 |
| Runs that updated the docs | 10 of 10 | 9 of 9 |
| `cargo test`, clippy, fmt clean | 10 of 10 | 9 of 9 |

**What it shows.**

- The harness changed nothing about acceptance. Every failure, in both arms, is
  the same held-out check: after raising the journal limit, a full journal of
  the largest allowed titles no longer fits the store's 256 KiB bound, so it
  cannot be saved. One stripped run noticed and asked; none fixed it.
- The harness's own CI did not catch it either: `cargo xtask check ci` passed
  on all nine harness runs, including the two that failed.
- A bare repository was enough for the habits the harness asks for. Every
  stripped run added tests, updated the driver doc and left `cargo test`,
  clippy and formatting clean, unprompted.
- The process was mostly skipped. One harness run of nine wrote a task record
  and ran `complete`; the other eight judged it out of proportion to the
  request and said so. That one run then left `docs/PRODUCT.md` un-updated,
  because editing it would have needed a receipt: the gate made the docs
  worse.
- The harness arm wrote somewhat more tests, and three of its runs added a
  test through the real binary, which the product skill asks for. Whether
  those tests catch anything the others miss was not measured.
- It cost about 2.7 times the wall-clock time. Part of that is this run's
  load: nineteen agents built at once, and `cargo xtask check ci` ran past the
  agents' two-minute tool limit, which several reported.

**What it does not show.**

- The edit-time and stop hooks did not fire: the runs were subagents working
  in clones, where the session's project hooks point at another repository.
  The hook that checks each edit is the part of the harness most likely to
  help and it is still untested.
- Headroom is thin. Four of five tasks were passed by every run in both arms,
  so the set separates the arms on one task only, and there it does not.
- Two runs per cell, one model, one small product. One harness cell has a
  single run: its second was blocked at launch and not retried.
- Subagents inherit this session's instructions, which point at an
  `AGENTS.md`. Several stripped runs looked for one, found none in their clone
  and said so; none read this repository's copy.
- It says nothing about work over many sessions or by several people, which
  is what task records, receipts and the archive are for. No single-task run
  can show that.

**Reading.** On single, well-specified product tasks the harness is cost
without measured benefit, and agents treat its record-keeping as optional.
What it might still earn is untested here: per-edit feedback, and coordination
over time. Until one of those is measured, an adopter who wants speed loses
nothing this run can detect by starting from the stripped layout and adding
the CI profile.

## The page reviewer, retired

This template once had an independent reviewer agent for task pages, calibrated
the way the guardrail section above describes: fresh reviewers judged pages
whose verdict was already known. Before the review brief had fixed checks, 19
of 21 runs gave the original verdict but missed specific defects one time in
three; with four fixed checks (consistent, provenance, cited, ticket) every
targeted defect was raised 3 of 3, and pages expected to pass did worse. The
reviewer was removed after the paired run above: its loop cost more than the
authoring and most findings were about the page, not the change. The four
checks survive as the author's own checklist in the author-artifact skill. The
reviewer, its brief and the calibration record are on the
`showcase/full-ceremony` branch, and the calibration set at the commit
`harness/archive.json` names.
