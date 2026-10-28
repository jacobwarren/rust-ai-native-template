# Plan: mandatory HTML task review artifacts

Status: completed implementation; visual review pending human observation. Owner: maintainers. Starting commit: f78c3c15e89902d3071308c56e4243dc4054c16e.

## Purpose and non-goals

Every active task presents unknowns before implementation, accumulating notes,
a buy-in brief and an interactive quiz in separate durable self-contained HTML pages, with a small index.
The page supports human understanding; it does not grant approval or replace CI.
No model service, frontend build, or external hosting is required.

## Context and orientation

Task validation lives in xtask/src/task_record.rs and completion.rs. Existing
schema v1/v2 records remain readable as history; active admission moves to v3.
The prior optional-HTML proposal is superseded by the user's mandatory requirement.

## Milestones and acceptance

1. Record a pre-implementation snapshot before code edits and expose an offline
   HTML page. Commands reject invalid inputs and preserve prior snapshots.
2. Append implementation notes, publish a buy-in brief and scenario quiz. Render
   deterministic HTML with escaped content, navigable stages and answer feedback.
3. Enforce artifacts in coordination and active completion/handoff. Missing or
   stale HTML and incomplete final stages fail. Partial handoffs retain honest
   pending stages; old unchanged records need no fabricated history.
4. Add public CLI regression tests for lifecycle, history, stale/missing artifacts,
   hostile content, invalid questions and completion integration. Run normal CI.

## Progress

- [x] Read contracts, existing parser, fixtures and baseline failure evidence.
- [x] Implement structured artifact commands, rendering and gates.
- [x] Validate positive/negative CLI paths and actual quiz handler with a stub DOM.
- [ ] Human visual/browser review: local-file preview was rejected by browser security policy; user received the populated HTML.
- [x] Finalize records; final CI and completion outputs are recorded under target/verification/html-tasks.

## Discoveries

Prior baseline runs failed during concurrent Git fixture initialization. Fixture
names use timestamps and process IDs without exclusive creation. Investigate and
fix isolation if it prevents reliable new regression checks; preserve the finding.

Resolution: fixture allocation now uses a process-local sequence plus exclusive
directory creation/retry. A 16-thread test independently checks unique directories
and marker isolation. Normal workspace tests now pass. Exact timestamp collision
was not instrumented in the failing run; the unsafe reuse mechanism is removed.

## Decisions

Use repository-owned JSON plus deterministic checked-in index.html under
harness/task-artifacts/ID. Commands preserve initial metadata and append notes
and brief revisions. Base-history comparisons reject rewriting prior entries.
Checks verify structural integrity and rendering, not when a human actually read
an artifact or whether the reasoning is correct. The initial snapshot records
source identity before artifact creation; it does not claim a final CI hash.

## Interfaces and dependencies

Add task-artifact commands to xtask; use existing serde/serde_json and SHA tools.
Schema v3 uses a conventional per-ID artifact location, preventing arbitrary
output paths. Optional enrichment can be added later; four HTML views are required.

## Recovery and idempotence

Render can repair missing/stale HTML from durable JSON. Record publication uses
atomic replacement; concurrent mutation must fail explicitly. Start refuses an
existing artifact; note and brief commands append rather than erase history.
Partial/blocked handoffs require the initial artifact but allow pending final
stages with task remaining actions. No deployment or remote action is involved.

## Evidence

Baseline and final reports go under target/verification/html-tasks. Tests assert
independent expected content and rejection behavior through the CLI. Browser
inspection exercises the actual generated HTML and quiz feedback.

## Outcomes and retrospective

Implemented all four mandatory standalone HTML artifacts, lifecycle commands, schema v3 admission,
append-only checkpoint history, deterministic rendering and strict quiz inputs.
Eight public CLI tests and existing harness tests passed. The actual quiz handler
passed its optional Node/stub-DOM check. Browser layout and native interaction
remain a separately disclosed manual qualification; no browser result is claimed.

## Revision notes

2026-10-28: created before implementation; user requires HTML for every task.

## Task reconciliation

Use harness/tasks/html-tasks.json and requirement task-artifacts. Reconcile the
prior review task into this feature's migration with explicit historical labeling.
Keep unrelated product gaps open. Finalize durable records before final CI.

## Verification scope revision

Browser security policy rejected file URL preview. Automated acceptance is the
public CLI lifecycle/admission boundary and the actual quiz handler with a stub
DOM. Manual visual qualification is tracked separately and is not represented by
these tests. The user has been given the complete HTML for that review.

## User visual feedback and layout revision

The user supplied a screenshot of the original combined document and requested
separate artifacts like the source examples. The screenshot shows visible legends
straddling quiz card borders and excessive spacing. The renderer now emits four
standalone pages plus an overview. Question headings sit inside cards, legends
are visually hidden but accessible, empty feedback takes no space, and navigation
is in normal document flow. Tests assert separate page content and rejection of
any missing page. This is an implemented correction, not a claim of successful
visual review of the revised layout.
