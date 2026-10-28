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
3. Choose the representation that makes this task's content clear: annotated
   excerpts, diagrams, before/after states, side-by-side alternatives, a
   step-through, toggles that change a visible consequence. Interactions exist to
   help the reader reason or reply. End with a way to send a reaction back, such
   as a reply assembled from their choices that they can copy.
4. Write a single self-contained HTML file at the output path. Link
   `foundation.css` first in the head as described in [the guide](../ARTIFACTS.md)
   and use its tokens. Inline your own layout CSS and scripts. Load nothing from
   the network.
5. Keep the page about the feature. No harness mechanics, hashes, revision
   talk or verification boilerplate. Report those to the caller instead.
6. Check your work. Open the page if you can render it, exercise each
   interaction, and compare it with the exemplar. If the exemplar would teach a
   reader more about its task than yours teaches about this one, revise.

Return the path, three lines on what the page lets a reader learn or decide, the
evidence it cites, and anything you could not check (rendering, interactions,
claims you could not verify).
