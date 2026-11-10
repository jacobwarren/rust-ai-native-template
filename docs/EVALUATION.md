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
