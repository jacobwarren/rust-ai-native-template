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

## Guardrail tests

A mechanical checker should have an allowed fixture and an intentional violation.
A semantic checker also needs benign near-misses, calibration, and a way to
abstain. Prefer advisory output until measured precision supports blocking CI.
Policy edits and grader edits should receive separate review from product changes
when possible. Local scripts are mutable; enforce authority in the hosting/CI layer.

## Calibrating the page reviewer

The artifact reviewer is a semantic checker, so it gets the calibration above.
Run fresh reviewers on pages whose verdict is already known and compare. The
set is next-prompt's three round-1 drafts that were revised, kept in
[harness/calibration/next-prompt](../harness/calibration/next-prompt/reference.round1.html)
(their SHA-256 match the round-1 rows of that task's reviews.json), and the
three committed pages that passed (`harness/task-artifacts/next-prompt/`).

Each run is a fresh artifact-reviewer with the current review-artifact skill.
Hand it the prompt the page itself quotes, not a value written into the record
later; a different prompt flips the verdict. Have it judge claims and the
ticket at the commit the page was reviewed at. Tell it to record no verdict in
any log; "do not run any command" makes reviewers skip rendering. Write the
expected verdicts down before any run:

| Page | Reviewed at | Expected | Why |
| --- | --- | --- | --- |
| reference draft | 8f1a6e6 | revise | `consistent`: the reply says Q5 stands; the table refines it |
| plan draft | cdaeb0f | revise | length, and earlier decisions listed again |
| buy-in draft | 1002bb0 | revise | `provenance`: main-side output typed but shown as captured; `cited`: "9 / 9 CI" names no source or command |
| reference page | cdaeb0f | pass | |
| plan page | 3c1aa8e | revise | `ticket`: the kept path required by the ticket is dropped with no reason |
| buy-in page | 17425b6 | revise | `consistent`: the fixed "nothing changes before merge" line (buy-in.html:306) and "one real reply" (:291) against two in the log; first run expected pass, changed after the result below |

The baseline, with the brief before the fixed checks (docs/plans/brief-calibration.md):
19 of 21 runs gave the original verdict. Length was caught every time. The
contradicting reply was caught 2 of 3, the typed output 2 of 3, the uncited
tile 1 of 3 and the dropped kept path 1 of 4. Opus at low effort gave the same
verdicts as Sonnet at a higher price.

With the fixed checks (the final wording of review-artifact step 2), every
targeted defect was raised every time: the contradicting reply, the typed
output, the uncited tile and the length each 3 of 3 on their drafts, and the
dropped kept path 3 of 3 on the plan page. Each round is in
docs/plans/brief-calibration.md. Pages expected to pass did worse: the
reference page passed 1 of 3 and the buy-in page 0 of 3. Read against the
files, the buy-in failures were mostly real defects every earlier review had
passed, hence its new row above. Known gaps:

- The reference page's scratch-probe blocks (`probe_guide`, a patched `Task`)
  fail `provenance` and `cited` 2 of 3 times. They are labelled scratch runs
  but show commands no reader can run, so the stricter reading is defensible;
  its row stays pass until an owner decides.
- `cited` is read more strictly than written: two of three buy-in runs failed
  tiles whose command sits in the sentence beneath them. The author brief asks
  for a source beside every tile, which meets either reading.
- `ticket` also fires on buy-in pages for ticket changes reported only as
  decisions, and once on a recorded assumption missing from the risks.

Rerun this whenever the review brief, the reviewer's model or tools, or the
review line of the hand-over (xtask/src/handover.rs) change. Three runs per
page is the smallest count that shows a miss rate. A check that stays below
3 of 3 is a known gap: record it here rather than tuning the brief to these
six pages, which would fit the brief to its own calibration set.
