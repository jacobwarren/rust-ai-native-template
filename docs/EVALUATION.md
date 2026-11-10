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

## Calibrating the page reviewer

The artifact reviewer is a semantic checker, so it gets the calibration above.
Run fresh reviewers on pages whose verdict is already known and compare. The
first calibration ran on the template's own history, which now lives at the
commit `harness/archive.json` names (branch `showcase/template-history`):
three round-1 drafts of the next-prompt task that were revised
(`harness/calibration/next-prompt/` there) and its three committed pages that
passed. Its method and every round are in `docs/plans/brief-calibration.md` at
that commit. What it found, and what the current review brief carries:

- Before the brief had fixed checks, 19 of 21 runs gave the original verdict,
  but a contradicting reply was caught 2 of 3 times, typed output shown as
  captured 2 of 3, an uncited tile 1 of 3 and a dropped ticket item 1 of 4.
- With the four fixed checks (consistent, provenance, cited, ticket) every
  targeted defect was raised 3 of 3. Pages expected to pass did worse, mostly
  for real defects earlier reviews had missed.
- `cited` is read more strictly than written, and `ticket` fires on changes
  reported only as decisions. These are known gaps, not tuned away.

To calibrate again, build a set from your own tasks: keep each reviewed draft
(`task-artifact verdict` saves one under `target/artifact-reviews/`), write the
expected verdict and its reason down before any run, and hand each fresh
reviewer the prompt the page itself quotes. Rerun whenever the review brief,
the reviewer's model or tools, or the review line of the hand-over
(xtask/src/handover.rs) change. Three runs per page is the smallest count that
shows a miss rate. A check that stays below 3 of 3 is a known gap: record it
rather than tuning the brief to its own calibration set.
