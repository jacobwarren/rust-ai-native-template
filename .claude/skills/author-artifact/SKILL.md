---
name: author-artifact
description: Write a task's brief, the one HTML page a standard task commits before its code, from its prompt and exemplar. Use when cargo xtask task-artifact next hands you a brief to write.
---

# Brief author

You are writing one HTML page that the task's owner reads before writing code:
what the ticket does not show about this part of the codebase, and a plan that
leads with the decisions most likely to change. You were given a prompt, an
exemplar page, the task context, the rules and an output path. That page is
your whole job; do not edit product code or commit anything.

1. Read the exemplar in `examples/thariq/` in full, both its source and, if you
   can, its rendered page. Notice what it lets the reader see, decide and send
   back. Your page does that job for this task. Its layout, domain, visual
   style and length are not the target.
2. Do the investigation the prompt asks for. Read the code, tests, history and
   docs the change concerns, as they are committed. Every unknown, option and
   claim needs a concrete basis the reader can follow: a file and line,
   function, test, commit or command output. Look hardest for what depends on
   the thing being changed and is not named in the ticket. When the honest
   answer is short, say so with its basis rather than padding.
   The task's owner may be editing this worktree while you work. Build and run
   anything in a clean copy (`git archive HEAD | tar -x -C DIR`, or a
   `git clone` where history is needed, in a scratch DIR under `target/` with
   its own `CARGO_TARGET_DIR`), not in the worktree, and remove it afterwards.
3. Choose the representation that makes this task's content clear: annotated
   excerpts, before/after states, side-by-side alternatives, toggles that
   change a visible consequence. Interactions exist to help the reader reason
   or reply. End with a reply assembled from the reader's choices that they
   can copy.
4. Write a single self-contained HTML file at the output path. Paste the shared
   style block first in the head as described in [the guide](../../../docs/ARTIFACTS.md)
   and use its tokens. Inline your own layout CSS and scripts. Load nothing,
   neither from the network nor from files beside the page.
5. Follow the rules in your hand-over. The file stays at or under 20 KB: check
   with `wc -c` before you hand back, and cut what a reader would learn as fast
   from the diff. Nobody will answer questions, so each question you would have
   asked goes on the page with your recommendation, labelled as the agent's
   call.
6. Hold yourself to four checks; nobody else will run them:
   - **consistent**: build the reply from the same data the page shows, so a
     choice, table row or tile and the reply can never disagree.
   - **provenance**: show output as captured only if you ran it and pasted it.
     Label typed, reconstructed or trimmed output, shortened excerpts and
     behaviour that does not exist yet as what they are.
   - **cited**: put a source beside every headline number and claim a decision
     rests on: a file and line, a test, captured output, or the command that
     reproduces it.
   - **ticket**: account for each acceptance, scope and validation item of the
     ticket: planned as written, or changed with its reason.
7. Check your work by looking at it. Render it with
   `node scripts/artifact-screenshots.cjs OUT_DIR YOUR_PAGE.html` (a scratch
   OUT_DIR under `target/`) and view the PNG tiles. Fix every console error,
   network request and phone-width overflow the JSON report lists. Exercise
   each interaction.

Keep the page about the change: no hashes, revision talk or verification
boilerplate. Report those to the caller.

Return the path and size, three lines on what the page lets a reader learn or
decide, the evidence it cites, anything in the task owner's working draft you
noticed the page contradicts, and anything you could not check.
