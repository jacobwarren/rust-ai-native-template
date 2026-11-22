# The task brief

Most tasks here write no page at all. A task that changes what every later
task is held to writes one first: a brief, a single HTML page that shows what
the ticket does not and where the plan could go wrong, before any code.

That is the whole ceremony, and it is this small on evidence. A paired run on
product features found no accepted change that task pages bought and fifteen to
forty times the wall-clock time; the pages that did find defects were briefs on
changes to the gates themselves ([evaluation](EVALUATION.md)). The earlier
eight-page set and its review loop are not in this template. The branch
`showcase/full-ceremony` names the last commit that has them.

## Task size

A task record carries `"tier": "small"` or `"standard"`; a record without a
tier is small. Choose at task start: a brief must be committed before the first
product-code commit, so a task cannot become standard after its code lands.

| Tier | Owes | Choose it when |
| --- | --- | --- |
| small | nothing | Any change, of any size, away from policy, dependency and ceremony paths. This is the default. |
| standard | `brief.html` before code | The change touches a path the gates protect, a `Cargo.toml` or the ceremony's own files, or is otherwise hard to undo: a persistence or wire format, a security boundary, a migration. |

Both tiers owe the task record, its four review answers, current CI evidence
and any transition receipt. The four review answers are the report on the
finished work; no page restates them.

The completion and handoff gates read the branch against its merge base and
refuse a small task when it touches a protected policy path other than the
product's own acceptance files, any `Cargo.toml`, or the ceremony's own
definitions (this guide, the author-artifact and write-ticket skills and the
`docs/skills/author-artifact.md` pointer, and the agent folders under
`.claude/` and `.codex/`). A mistake there changes what every later task is
held to, so those changes owe a brief. Size alone never does: a 600-line
feature with tests is a small task. Neither do `harness/workflows.json`,
`harness/scenarios.json`, `harness/readiness.json`, `harness/goldens/` and
`docs/PRODUCT.md`: a feature adds its workflow case and its tests there, and
those edits keep their transition receipt (and a golden its approval) without
a brief.

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

A small task that turns out to need a protected path is split, or restarted as
a standard task on a new branch with its brief first. The path list lives in
`xtask/src/size.rs`; changing it needs a transition receipt.

A record below schema version 7 is history: it can be read, owes nothing and
cannot make a new claim. Records written while this template had a full tier
may carry a `reader` value and extra `prompt` names; both are accepted and
ignored.

## Writing the brief

`cargo xtask task-artifact next --task ID` says the one next step: write the
record, write the brief, commit it, build, or complete. For a brief it prints a
`handover` to give the author unchanged. The prompt, with its braces filled
from the record's `prompt.task` and `prompt.area`:

> I'm about to {task} in {area}. Before I write code, give me one page: the
> unknowns in this part of the codebase that the ticket does not show, each
> with the code that proves it, then the plan, leading with the decisions I'm
> most likely to tweak and burying the mechanical work at the bottom.

and the rules that keep it short and self-sufficient:

> Keep the file at or under 20 KB, the pasted stylesheet included: five
> unknowns at most, each proven by code or captured output, and one decision
> per screen. No person will answer questions before the code is written, so
> put each question you would have asked on the page with your recommendation,
> labelled as the agent's call.

`unfilled` names any `prompt` value the record lacks, whose brace stays visible
in the text. `next` advises and never blocks; it exits 3 when it has a warning,
such as a brief over 20 KB.

1. **Use a dedicated author.** Start one fresh subagent with the
   [author-artifact brief](../.claude/skills/author-artifact/SKILL.md) and the
   `handover`. An author that has not been writing the change reads the code
   as it is, which is where the briefs that paid off found their defects. If
   your host has no subagents, give the brief a turn of its own.
2. **Read the exemplar first.** The brief is modelled on Thariq Shihipar's
   [tweakable plan](../examples/thariq/08-implementation-plan.html), from his
   "Know your unknowns" pages. Match the job its interactions do, not its
   layout, domain or length.
3. **Ground every claim.** Name the file, function, test or commit behind each
   unknown, and label output as captured only when it was run and pasted.
4. **Look at it.** `cargo xtask task-artifact screenshot --task ID` renders the
   page with local Chrome at desktop and phone width into
   `target/artifact-screenshots/ID/` and reports console errors, phone-width
   overflow, network requests and any file the page loads from disk. Without
   Node or Chrome, say so rather than implying visual review.
5. **Act on it, then commit it.** The task owner reads the brief, checks what
   it found against the code, changes the plan where the brief is right, and
   commits the page before any product code. One author pass is the whole
   loop. There is no reviewer and no rewrite: in the paired run the review loop
   cost more than the authoring, and most of its findings were about the page,
   not the change. Record what the brief changed in the task record's
   `discoveries`.

Keep the brief about the change. Leave out commit hashes, verification
boilerplate and paragraphs about what the page is not.

## Files and commits

The harness checks presence, a floor of text and order. Whether the brief is
useful is the owner's call.

1. After creating the task record, run `cargo xtask task-artifact style --task ID`.
   It creates `harness/task-artifacts/ID/` and writes the shared stylesheet to
   `target/artifact-style.html` as a `<style data-artifact-foundation="1">`
   block. Paste that block first in the brief's head and use its tokens for
   color, type and spacing.
2. Write `harness/task-artifacts/ID/brief.html` as one self-contained file that
   loads nothing else, so it renders the same in a file preview, on a code host
   or as an attachment. It holds at least 1,200 characters outside its tags and
   styles; a page that is only the stylesheet is refused.
3. Commit the brief before the first product-code commit. Completion and
   handoff read the branch's own commits since the base and check that order,
   for a partial task too. Code and brief must be committed when you claim
   completion. Nothing is owed after the code, so a late fix touches no page.

On a pull request a code host checks out a merge commit it built, from which
the branch's own commits cannot be read. CI passes the branch tip to
`cargo xtask handoff --base REV --head REV` so the order is checked there too.
Without `--head`, a merge commit built after the target moved can make a
correctly ordered branch look late; the error says to pass it.

`cargo xtask task-artifact guard --task ID` fails until a brief the task owes
is committed. The opt-in [Codex example](../examples/codex-artifact-hooks.json) runs it
before `apply_patch` edits to product code. It does not cover shell or MCP
writes. The default stop hook gates completion claims; see
[COMPLETION.md](COMPLETION.md).

The exemplar is from Thariq Shihipar's
[html-effectiveness](https://github.com/ThariqS/html-effectiveness), vendored
unmodified under Apache-2.0; see [its provenance](../examples/thariq/README.md).
