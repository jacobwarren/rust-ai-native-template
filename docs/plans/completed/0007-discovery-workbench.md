# Plan: interactive pre-implementation discovery

Status: completed implementation. Owner: maintainers. Base: f78c3c15e89902d3071308c56e4243dc4054c16e.

## Purpose and non-goals

Replace generic unknowns prose with five mandatory standalone decision tools:
blindspot pass, intervention brainstorm, interview, reference semantics map and
tweakable plan. Humans can explore and export choices for explicit import into
an append-only record. Browsing is not approval; no server or model is required.

## Context and orientation

The existing Before strings in xtask/src/task_artifact.rs cannot model choices,
questions or matched semantics. Preserve that initial snapshot while adding
versioned discovery revisions and feedback. Archived v3 records remain readable;
new active claims use v4. Existing post-implementation artifacts remain required.

## Milestones and acceptance

1. Add strict structured discovery data and five distinct HTML interactions.
2. Export page-specific feedback bound to task, discovery and feedback revision;
   import only valid current responses, append history, and retain original data.
3. Gate v4 claims on all five artifacts, current publication and blocking answers.
   Explicitly sourced settled answers and honest no-reference states are allowed.
4. CLI regressions reject stale/wrong-task/invalid feedback and rewritten history;
   JS behavior checks verify selection summaries, interviews and preview changes.
   Run full CI and completion; disclose unverified browser layout.

## Progress

- [x] Inspect current harness and source examples; record preflight before edits.
- [x] Implement model, renderer, feedback and admission.
- [x] Exercise CLI and actual generated UI code in a DOM implementation, including real CLI import.
- [x] Finalize docs and records; final reports are under target/verification/discovery-workbench.

## Discoveries

The previous artifacts were summaries, not tools for resolving uncertainty. A
single scalar alternatives field cannot support live comparisons or human input.
Local HTML cannot silently write repository files; feedback needs explicit import.

## Decisions

Use typed data and fixed offline renderers. Do not permit task-authored scripts.
Retain immutable preflight and append discovery/feedback revisions. Feedback is
bound to content hashes, not an embedded self-referential workspace fingerprint.
Imported changes invalidate the existing final brief until republished.

## Interfaces and dependencies

New discovery model, validation, renderer and JavaScript modules in xtask. Use
existing serde/serde_json/hash helpers. New task-artifact discover and decide
commands extend the existing lock and atomic publication protocol.

## Recovery and idempotence

Rejected inputs leave records unchanged. Render repairs interrupted page writes.
Browser drafts are proposals until exported and imported. A new discovery revision
makes old feedback stale; never silently replay old choices against changed options.

## Evidence

Reports under target/verification/discovery-workbench. Use public CLI tests with
independent expectations and reproducible optional JS tests. Browser file-preview
was previously blocked by policy; no circumvention or false visual claim.

## Outcomes and retrospective

Implemented five distinct offline discovery tools, task schema v4, artifact
schema v2, revision-checked exports/imports, preserved histories and current-input
publication gates. Four new Rust CLI regressions and the existing artifact tests
pass. Generated UI integration passes through actual browser-side event handlers
and real CLI imports in an isolated workspace. Native browser layout, clipboard
permissions and file-download behavior remain explicitly unverified.

## Revision notes

2026-10-28: user requires the five discovery artifacts as interactive tools.

## Task reconciliation

Task discovery-workbench; requirement discovery-workbench. Existing product gaps
remain outside scope. Complete the four review questions and source-bound gates.

## Material discoveries and limits

An artifact can be structurally present yet remain a prose summary. Typed fields
and interaction tests now cover the actual decision workflows. A second finding
was that note counts alone do not establish a current brief: discovery and feedback
hashes now participate both in admission and visible publication status. The plan
artifact assembles imported decisions and sourced answers from all five tools.
It does not autonomously rewrite code or claim user approval.
