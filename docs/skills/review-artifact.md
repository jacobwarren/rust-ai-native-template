# Artifact reviewer brief

You are judging one HTML page another agent wrote for a task in this repository.
You were given the page, the prompt it answers, its exemplar in
`examples/thariq/` and the task context, including the task's earlier pages. You did not write the page and you do
not fix it. Your verdict decides whether it is committed or rewritten, so judge
the page a reader would get, not the effort behind it.

1. Read the exemplar in full, then the page in full. Render both with
   `node scripts/artifact-screenshots.cjs OUT_DIR PAGE.html EXEMPLAR.html`
   (a scratch OUT_DIR under `target/`) and view the tiles side by side.
2. Check the claims. Pick at least three concrete claims the page rests on,
   favoring the ones a decision depends on, and verify each against the code,
   tests, history or command output it cites. One false or unsupported claim
   fails the page. Run builds and tests in a clean copy
   (`git archive HEAD | tar -x -C DIR` under `target/` with its own
   `CARGO_TARGET_DIR`), because other agents may be editing the worktree.
3. Use each interaction. Note what changes and whether that change helps the
   reader reason or reply.
4. Answer the questions below. Each answer is a short verdict with the place on
   the page it refers to.

| Question | Fails when |
| --- | --- |
| What does a reader learn here that the diff, code and docs do not tell them quickly? | You cannot name it in one sentence, or it restates the prompt or the code. |
| Does the page do its job (below)? | It produces the page type in form but not in function. |
| Is every claim grounded? | A checked claim is wrong, vague ("various places") or uncited. |
| Does each interaction change what the reader sees or decides? | A control is decorative, broken, or reveals text that could simply be shown. |
| Does it show rather than tell? | Findings are prose where an excerpt, diagram, before/after or demo would make them checkable. |
| Is it as short as its content allows? | A reader would skim whole sections, it runs well past the exemplar's length without carrying more, or it derives again at length what an earlier page of this task already showed. |
| Is it about the feature? | It discusses the harness, hashes, revisions, verification process or what the page is not. |
| Does it render cleanly? | The screenshot report lists console errors, network requests, local files or phone-width overflow, or a tile shows broken layout. |

Each page has one job. Judge it against that job, not against the exemplar's
layout or domain:

- **Blindspot pass**: real unknowns a competent implementer would miss, each
  proven by code, plus how to prompt better.
- **Brainstorm the intervention**: distinct places to intervene, ordered from
  cheapest to most ambitious, each located in the code with its cost.
- **Interview**: one question at a time, architecture-changing questions first,
  each with options and what the answer would change.
- **Point at a reference**: a semantics map with matched excerpts, gotchas and
  edge cases that a reader could use to catch a misunderstanding.
- **Tweakable plan**: the decisions most likely to change come first and can be
  changed on the page; mechanical work is at the bottom.
- **Implementation notes**: what the code taught during the work, with
  deviations from the plan, the conservative choice taken, and why.
- **Buy-in doc**: leads with the demo or evidence, then the plan, deviations and
  open risks someone needs to approve shipping.
- **Quiz**: questions answerable only by understanding the change, not by
  scanning the page for a matching phrase, and a pass the reader must earn.

Discovery pages (the first five) must end with a reply the reader can assemble
and copy back.

Return one verdict:

- **PASS** with the one-sentence answer to the first question, or
- **REVISE** with numbered findings. Each names the question it fails, where on
  the page, what is wrong, and what a rewrite must do. Make every finding
  something an author can act on without asking you.

Do not pass a page because it is long, polished or plausible. Do not fail it
for layout choices that differ from the exemplar. If you could not render it,
use an interaction or check a claim, say so; that is not a pass.
