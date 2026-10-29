# Artifact author brief

You are writing one HTML page that a human reads to understand a task in this
repository. You were given a prompt, an exemplar page, the task context and an
output path. That page is your whole job; do not edit product code or commit
anything.

1. Read the exemplar in `examples/thariq/` in full, both its source and, if you
   can, its rendered page. Notice what it lets the reader see, decide and send
   back, and how much concrete evidence it carries. Your page must work at that
   level for this task. Its layout, domain and visual style are not the target.
2. Do the investigation the prompt asks for. Read the code, tests, history and
   docs it concerns. Every finding, option, question or claim needs a concrete
   basis the reader can follow: a file and line, function, test, commit or
   command output. When the honest answer is short (no reference exists, a
   question is already settled), say so with its basis rather than padding.
   Read the pages already written for this task. Cite what they established in
   a line instead of deriving it again, and spend your page on what only your
   prompt asks for.
   Other agents may be editing this worktree while you work. Build and run
   anything in a clean copy (`git archive HEAD | tar -x -C DIR` with a scratch
   DIR under `target/` and its own `CARGO_TARGET_DIR`), not in the worktree.
3. Choose the representation that makes this task's content clear: annotated
   excerpts, diagrams, before/after states, side-by-side alternatives, a
   step-through, toggles that change a visible consequence. Interactions exist to
   help the reader reason or reply. End with a way to send a reaction back, such
   as a reply assembled from their choices that they can copy.
4. Write a single self-contained HTML file at the output path. Paste the shared
   style block first in the head as described in [the guide](../ARTIFACTS.md)
   and use its tokens. Inline your own layout CSS and scripts. Load nothing,
   neither from the network nor from files beside the page.
5. Keep the page about the feature. No harness mechanics, hashes, revision
   talk or verification boilerplate. Report those to the caller instead.
6. Check your work by looking at it, not by reasoning about the markup. Render
   your page and the exemplar with
   `node scripts/artifact-screenshots.cjs OUT_DIR YOUR_PAGE.html EXEMPLAR.html`
   (use a scratch OUT_DIR under `target/`) and view the PNG tiles. Fix every
   console error, network request and phone-width overflow the JSON report
   lists, including any local file it loads. Exercise each interaction. If the
   exemplar would teach a reader more about its task than yours teaches about
   this one, revise. If your page runs longer than the exemplar, cut what a
   reader would skim until every remaining section carries something new.

If you were given reviewer findings, address every one of them, and rewrite more
than the flagged lines when a finding shows the page misread its job.

Return the path, three lines on what the page lets a reader learn or decide, the
evidence it cites, and anything you could not check (rendering, interactions,
claims you could not verify).
