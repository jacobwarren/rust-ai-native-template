# Ordered artifact checkpoints

The first fresh-agent trial created discovery before code, but wrote notes with
its final brief and included an inert plan dropdown. New authored ledgers use
schema 2 to reject that shortcut. New task records use schema 5 and cannot
complete through the legacy fixed renderer or schema-1 authored ledger. Legacy schema-1 ledgers remain readable; their
history is not relabeled as proof of this workflow.

## Required sequence

1. Before implementation, register the five discovery purposes, with a working
   plan and its tested response mechanism. Do not include implementation notes,
   buy-in or quiz in this first registration.
2. After the first implementation slice or material discovery, **stop editing
   product code, append HTML implementation notes and register a new revision**.
   Include the five discovery pages and notes, but no buy-in or quiz. Record what
   changed, the evidence observed, the decision made and what remains next. Do this
   while work is ongoing, not after all implementation/testing is finished.
3. Repeat the implementation checkpoint at material milestones. If code or tests
   change, update notes before the next checkpoint. Preserve prior snapshots.
4. Before publishing final artifacts, make the last implementation checkpoint
   cover current product code, tests and contracts. Then add buy-in and quiz and
   register the final bundle. The final phase requires a preceding separate notes
   revision covering those same work bytes and the same notes bytes.
5. Finalize task administration and run full CI, completion and handoff.

The phase is inferred from purposes; no extra phase flag is needed. Registration
rejects final-only first bundles, discovery after implementation, unchanged-work
implementation checkpoints, changed work with unchanged notes, and final artifacts
without a current preceding implementation checkpoint. Completion rejects stale
work and non-final checkpoints. Rework after final publication uses a new
implementation checkpoint, followed by another final revision.

Work identity includes repository source except task artifacts, task records,
transition receipts, execution plans, the documentation catalog and gap register.
Those administrative edits do not pretend to be implementation progress. Other
code, tests, scripts, acceptance configuration and product documentation count.
A source hash proves which bytes were observed; it is not a trusted timestamp.

## A plan must do something consequential

Expose at least two real alternatives. Selecting one must change a visible
behavior preview, data-flow explanation, acceptance scenario, execution order or
other consequential result. Provide a usable copy/export response describing the
selected choice and its consequences. A dropdown labeled “review emphasis” that
changes nothing is not a tweakable plan. Do not invent an irrelevant alternative
just to satisfy a count; use a real product behavior or implementation tradeoff.

Layout, controls, diagrams, content and response formatting remain freely authored.
Read the [quality guide](ARTIFACT-AUTHORING.md) and the
[worked decision model](../examples/artifact-studies/plan.html): its controls change
visible state, driver error, recovery guidance and acceptance together. Its
[interaction tests](../examples/artifact-studies/test-studies.cjs) assert those
consequences and responses. These are optional patterns, not required page shapes.
The older [minimal wiring example](../examples/tweakable-plan.html) and its
[DOM test](../examples/test-tweakable-plan.cjs) are optional patterns, not templates
that the page must match. They demonstrate event/report plumbing only, not the
artifact quality bar. Adapt tested choices and explanations to the actual feature.

Exercise the actual handlers for at least two choices and assert independent
expected outcomes and responses. Save the result as a supporting JSON file inside
the artifact bundle and set `plan_evidence` in the registration input to its path:

```json
{
  "schema_version": 1,
  "plan_sha256": "SHA256-OF-THE-TESTED-HTML",
  "test_command": "the command actually run",
  "cases": [
    {"choice": "A", "before": "initial output", "after": "expected consequence A", "response": "exported choice A"},
    {"choice": "B", "before": "initial output", "after": "expected consequence B", "response": "exported choice B"}
  ]
}
```

Both cases must change an output, have different choices and consequences, and
produce nonempty responses. Start each case from another state if necessary.
Register the report with purpose supporting; retain it in every subsequent bundle.
Rerun it whenever the plan changes. Self-contained inline scripts are easiest to
bind to the plan hash. For external dependencies, register them too and rerun tests
when they change. The harness does not execute arbitrary report commands or prove
that assertions ran honestly. Review the test and its dependency coverage.

Example optional DOM setup (normal Rust checks do not require Node):

```sh
npm install --prefix target/artifact-dom --no-save --package-lock=false jsdom@30.1.3
NODE_PATH=target/artifact-dom/node_modules node examples/test-tweakable-plan.cjs PATH-TO-PLAN PATH-TO-REPORT
```

## Hook boundaries and limits

`cargo xtask task-artifact guard --task ID --point before-edit` requires a current
registered checkpoint. After a code edit, `--point after-edit` fails until notes
are updated and registered. `--point before-final` requires a current implementation
checkpoint. These commands never generate prose or repair an artifact themselves.
They give an agent a blocking diagnostic that tells it what to author next.

The opt-in [Codex hook configuration](../examples/codex-artifact-hooks.json) uses
[the write adapter](../scripts/agent-artifact-write.sh) for apply_patch PreToolUse
and PostToolUse, and [the completion adapter](../scripts/agent-complete.sh) for
Stop and SubagentStop. Build xtask first. Set HARNESS_TASK_ID, HARNESS_BASE and
HARNESS_REPORT in the host session environment, then copy the reviewed config to
the host's project hook location and review/trust it through the host UI. This
repository does not silently install hooks or grant trust to its own code.

The patch adapter permits artifact/administrative patches so the agent can repair
missing checkpoints. Source patches are guarded. Mixed patches are source patches.
Moves and absolute paths within the repository are checked; unrecognized or unsafe
paths are rejected. **Shell commands, MCP writes, existing terminal sessions and
other tools are not covered by this adapter.** Add and test adapters for every
write-capable path before claiming write-order enforcement. A PostToolUse denial
cannot undo a write that already happened.

Official documentation describes [hook events and trust](https://learn.chatgpt.com/docs/hooks)
and [managed enforcement limits](https://learn.chatgpt.com/docs/enterprise/managed-configuration).
Hook timeouts, callback errors or malformed responses may allow calls to proceed;
non-managed hooks are skipped until trusted. Stop blocks request continuation,
not proof of eventual completion. Repository code and self-authored evidence are
modifiable by the agent. Stronger enforcement requires an independently controlled
verifier/host and protected CI checks, with coverage and failure behavior tested.

There is no absolute guarantee of generation, usefulness or chronology from prompts
or local hooks. The enforceable local contract is: a checked schema-2 completion
cannot pass without ordered, content-bound records and plan scenario evidence.
An independent observer can verify actual sequence in a trial. Human review still
judges semantic usefulness, and browser review judges visual quality.
