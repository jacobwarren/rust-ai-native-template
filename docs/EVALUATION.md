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
