# Shared artifact styling

All newly registered authored HTML uses the shared foundation in
[artifact_foundation.css](../xtask/src/artifact_foundation.css). It supplies colors,
typography, spacing tokens, native controls, focus indicators, responsive base
spacing, reduced motion and print defaults. Layouts, diagrams, content and interaction
logic remain task-specific. No JavaScript library or external font is required.

## Authoring workflow

After creating the task record, install the current foundation into its working bundle:

```sh
cargo xtask task-artifact style --task ID
```

This explicitly writes harness/task-artifacts/ID/foundation.css. It never edits HTML
or historical snapshots. Running it again is idempotent for the same foundation;
a newer foundation updates the working copy and requires a new registration.
The command uses the same exclusive task lock as registration.

Place this exact link in each HTML document's head, before page-specific styles:

```html
<link rel="stylesheet" href="foundation.css" data-artifact-foundation="1">
```

For a page at tools/plan.html use href="../foundation.css"; add one ../ for each
nested directory. Keep the attribute order and double quotes shown above: the
bounded admission check checks this literal inclusion, not a full HTML parser.
It does not prove the link is active or the cascade is correct. Do not hide the
link in a comment, disable it or override the base tokens to circumvent the style
contract. Browser review establishes actual appearance.

Author page-specific CSS after the link. Use foundation variables for colors,
fonts, spacing, borders and radii. Prefer custom CSS for geometry: grid placement,
column proportions, diagram dimensions, responsive layout and interaction states.
Intentional domain colors (for example, color grading or syntax illustrations)
can differ; reuse foundation colors for the surrounding controls and prose.

```css
.approach-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(min(100%, 18rem), 1fr));
  gap: var(--space-6);
}
```

The optional [page starter](../examples/artifact-page.html) shows the link and a
custom layout. Copy and adapt it, or author a completely different document.
Optional foundation classes include surface, callout, warning, muted, eyebrow,
lead, secondary (buttons), and table-scroll. No class or particular DOM layout is
required. Apply aria-current="page" to the active navigation link.

## Registration and frozen presentation

Register the ordinary manifest with task-artifact register. Every listed HTML file,
including supporting pages, needs its correctly relative local foundation link.
The installed CSS must match this build's foundation. Registration automatically
adds foundation.css as a supporting asset; do not list it yourself. All other
supporting assets remain explicit manifest entries.

The existing snapshot mechanism copies that CSS into authored-revisions/N/ beside
the HTML, preserving nested relative links. Both HTML and CSS hashes are recorded.
Opening a historical HTML file therefore loads that revision's CSS, not the task's
working CSS or a repository-global file. A changed or missing saved CSS file fails
integrity checks. Do not link historical pages to mutable global stylesheets.

A foundation update must bump the version comment, --artifact-style-version and
VERSION in artifact_style.rs. Run style, review the working pages, then register a
new revision. Historical revisions are compared with their saved hashes, never
with today's foundation. Existing pre-foundation ledgers remain readable; every
new registration adopts the current foundation. Older fixed-renderer artifacts
remain frozen under their existing workflow. To use one as a starter for new work,
add the local foundation link, remove duplicate base styling, and register it as
authored HTML. Do not silently rewrite historical artifacts to adopt new styling.

## Verification boundaries

CLI regressions cover asset inclusion, nested relative paths, missing/altered CSS,
missing links, task locks, symlinks and style upgrades preserving old bytes and
custom HTML. Integrity is not a claim of visual quality or CSS semantics. Check
representative pages at narrow and wide widths, keyboard focus, long headings,
form controls, diagrams and exported responses. Record actual checks and unrun
visual review in each artifact's verification metadata.

Reproduce the optional DOM checks with jsdom available on NODE_PATH:

```sh
node scripts/test-artifact-style.cjs
```

The script checks working and saved example pages, local CSS resolution, CSS parsing
and preserved interactions. It does not perform browser visual verification.
