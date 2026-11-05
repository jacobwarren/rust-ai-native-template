---
name: write-ticket
description: Turn a request into a ticket this harness can check (Problem, Scope, Acceptance as named tests, Validation, Out of scope) and a partial task record stub under harness/tasks/. Use when asked to write, draft or refine a ticket, or before starting work that has no task record.
---

# Write a ticket

A ticket here fixes scope, tier and acceptance before work starts, in terms the
gate can check later. Write it, then write the task record stub from it. Do not
start the work; the ticket is the hand-off.

## 1. Read before writing

- The request, in the owner's words. Keep their words for the problem.
- `harness/gaps.json`: name any open gap the work touches.
- `harness/readiness.json`: the requirement ids the work proves (`requirements`
  in the record). An id must already exist there; a new requirement is a policy
  change that needs its own receipt.
- The code the change lands in. Cite files, not guesses.

## 2. The ticket

Write these five sections, short and concrete:

1. **Problem.** What is wrong or missing and who notices, with the evidence: a
   command and its output, a file and line, a failing case.
2. **Scope.** The files or areas that change, and the tier:
   - **small** when the change is at most 100 changed lines and touches no
     protected path (policy, gates, goldens, hooks), no `Cargo.toml`, and none
     of the ceremony's own files: skills, agents, `docs/ARTIFACTS.md`. Small
     tasks need no pages; the gate measures the branch and refuses one that
     grows.
   - **standard** otherwise. It owes the eight task pages, discovery before
     code (docs/ARTIFACTS.md). When unsure, say standard.
3. **Acceptance as named tests.** One line per behaviour:
   `test_name_in_snake_case: given …, when …, then …`. Name tests that do not
   exist yet; these names become the record's remaining actions now and its
   `readiness.json` tests when the work lands.
4. **Validation.** The commands that show it works: always
   `cargo xtask check ci > target/verification/ci.json`, plus any workflow,
   scenario or manual step, with what to look for.
5. **Out of scope.** What a reader might expect that this work will not do, and
   where it goes instead (a gap, a later ticket).

## 3. The record stub

Write `harness/tasks/<id>.json` from the ticket. The record is strict (unknown
fields fail), there is no `planned` status, and a partial record must list its
remaining actions:

```json
{
  "schema_version": 6,
  "id": "<id>",
  "work_item": "<tracker URL, or null>",
  "status": "partial",
  "tier": "small | standard",
  "requirements": ["<readiness ids from Scope>"],
  "related_gaps": [{"id": "<gap>", "disposition": "unaffected", "rationale": "<why>"}],
  "gap_review": "<which gaps were read and why they are or are not touched>",
  "discoveries": [],
  "discovery_review": "Not started; discoveries are recorded as the work finds them.",
  "documentation": {"updated": [], "rationale": "<docs the work will change, or why none>"},
  "prompt": {
    "task": "<verb phrase that reads after I'm about to>",
    "task_noun": "<noun phrase naming what is built; reads after build and shipping>",
    "area": "<the code area, for the blindspot pass>",
    "problem": "<the Problem, with no final period>",
    "reference": "<one code path that already behaves this way, singular>"
  },
  "summary": "<Problem, one sentence>",
  "remaining": ["<one per named test: test_name: given …, then …>", "<Validation commands>"]
}
```

A standard task fills `prompt`: `task-artifact next` puts these values into
each page's prompt (docs/ARTIFACTS.md), and a value left as its `<…>`
placeholder or blank stays a visible brace. A small task can drop `prompt`.

`id` matches the file name (letters, digits, `-`, `_`). Leave `review` out until
the work is done; before a PR, handoff asks the record its four review
questions (acceptance, verification, assumptions, coverage), so answer them
then. Requirement ids are checked at handoff too, not by `coordination`.

After writing the stub, run:

- `cargo xtask coordination`: the stub must parse and validate.
- `cargo xtask task-artifact next --task <id>`: the first step of the work.

Put the ticket itself where the owner tracks work. When there is no tracker,
put it in the PR description, or in `docs/plans/<id>.md` (an administrative
path).
