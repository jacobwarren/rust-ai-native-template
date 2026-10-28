# Plan: task-authored HTML without fixed templates

Status: completed. Owner: maintainers. Base: f78c3c15e89902d3071308c56e4243dc4054c16e.

## Purpose and non-goals

Agents author HTML/CSS/SVG/JavaScript for the actual task. Require the five discovery
purposes and later notes, buy-in and quiz, not a fixed layout or response schema.
Keep existing renderers as optional starters; never overwrite authored HTML.

## Context and orientation

Current task_artifact checks compare renderer output byte-for-byte. Add a separate
authored manifest/ledger with registered files and immutable snapshots. Admission
uses that contract when present; historical template records remain supported.

## Milestones and acceptance

1. Register freely authored HTML with purpose/evidence/feedback/verification metadata.
2. Preserve bundle snapshots and reject unregistered changes, missing purposes,
   invalid paths, stale feedback and rewritten committed history.
3. Accept arbitrary feedback payloads inside a revision-bound envelope.
4. Prove two different layouts pass without renderer equality; verify that legacy
   render commands cannot overwrite authored files. Run full CI and completion.

## Progress

- [x] Inspect current contracts and record baseline/preflight.
- [x] Implement registry, snapshots, feedback and gates.
- [x] Add independent regression tests and update authoring instructions.
- [x] Finalize source records, CI and completion.

## Discoveries and decisions

The invariant is reviewable purpose and provenance, not renderer output. Hashes
bind a registered revision to its actual bytes; they do not prescribe those bytes.
Custom JavaScript is authored code requiring review, not automatically sandboxed.

## Interfaces and dependencies

Add task-artifact register and authored feedback routing with existing Rust deps.
Use authored.json alongside preserved legacy record.json. No forced migration of
historical snapshots. Each registered revision contains the complete current bundle.

## Recovery and idempotence

Append revisions under an exclusive lock; failed ledger publication can leave an
orphan snapshot directory, which can be inspected and removed before retrying.
Never regenerate arbitrary HTML. Checks only read and compare registered hashes.

## Evidence and outcomes

Reports under target/verification/authored-html. Five independent CLI tests and strict Clippy passed. Authored example DOM checks cover response exports and alternative/plan consequence controls. Final full CI and completion are required at handoff. The initial fast checks passed individually but the report was invalidated by concurrent source edits; it is not claimed as a valid baseline. Semantic
usefulness, JS safety and browser layout remain human review responsibilities.

## Task reconciliation

Task authored-html; registered acceptance authored-html. Existing product gaps
remain outside scope. Preserve all preceding task histories and policy rationale.
