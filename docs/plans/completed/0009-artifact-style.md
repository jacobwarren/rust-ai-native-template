# Plan: shared artifact styling with historical isolation

Status: completed. Owner: maintainers. Base: f78c3c15e89902d3071308c56e4243dc4054c16e.

## Purpose and non-goals

Give all new authored pages consistent typography, colors, spacing and controls.
Authors retain task-specific layouts, content and interactions. Bundle CSS in every
registered revision; a future foundation update must never restyle prior snapshots.

## Milestones and acceptance

1. Ship a versioned CSS foundation and explicit style installation command.
2. Registration requires local foundation links and snapshots the CSS automatically.
3. Verify missing/altered CSS and missing links fail; nested pages resolve locally;
   historical CSS bytes survive style upgrades and unregistered edits fail.
4. Update author instructions and demonstrate different layouts with shared styling.

## Progress and evidence

Baseline fast passed before edits. Reports: target/verification/artifact-style.
Five pre-implementation pages registered before implementation. Eight CLI regressions,
strict Clippy, docs checks and 38 current/historical DOM page checks passed.
Full CI and completion reports are produced after source finalization in the report directory.

## Decisions and recovery

Use local CSS assets, not repository-global links. The style command explicitly
updates only the task working copy; existing snapshots remain immutable. Do not
rewrite historical tasks or require old revisions to match the current foundation.
CSS supplies base tokens and elements; page CSS remains allowed for composition.
Preserve existing lock and atomic ledger semantics. Registration snapshots full bundles.

## Task reconciliation

Task artifact-style. No existing product gap is closed. Acceptance: artifact-style.
Visual quality and CSS override semantics require review, not substring-based proof.

## Outcomes and remaining uncertainty

Shared tokens and native elements are bundled automatically at registration. The
style install command changes only the working copy. An independent simulated
prior-release fixture proves that a new foundation leaves historical bytes intact.
Custom HTML and interactions remain unchanged by installation. Existing authored
examples adopt the foundation in a new revision, preserving previous snapshots.
Browser visual qualification remains a human follow-up; no new unresolved product
gap was found in the inspected registry and style lifecycle.
