# Plan: <observable outcome>

Status: draft. Owner: <owner>. Starting commit or source fingerprint: <identity>.

This is a living execution plan. Keep Progress, Discoveries, Decisions, and
Outcomes current at every stopping point. A new agent must be able to resume
using this plan and the repository, without prior conversation.

## Purpose and non-goals

Explain who benefits, what they can do after the change, and how to observe it.
State limits. Define unfamiliar terminology where it first appears.

## Context and orientation

Name exact repository-relative files, important symbols, current behavior, and
relevant contracts. Explain how the pieces connect. Embed required facts and
assumptions; do not rely on an external conversation or an unread prior plan.

## Milestones and acceptance

For each milestone, describe the work, resulting observable behavior, exact
commands with working directory, expected output, and failure interpretation.
Each milestone should be independently verifiable. Include an independent
regression reproducer where behavior changes. Name the test that fails before
and passes after; compilation alone is not acceptance.

## Progress

- [ ] <timestamp> Establish and record the baseline.
- [ ] <timestamp> Complete the first observable milestone.
- [ ] <timestamp> Verify affected behavior, boundaries, and docs.
- [ ] <timestamp> Review evidence and record remaining uncertainty.

Split partially completed steps into finished and remaining work. Record the
next exact action before stopping, including relevant artifact paths.

## Discoveries

Record unexpected behavior, failed hypotheses, relevant tool/library behavior,
and evidence. Explain whether each observation changes the plan.

## Decisions

For each consequential decision: date/author, choice, alternatives, rationale,
and the evidence that supports it. When changing course, update all affected
sections so this remains one coherent current plan.

## Interfaces and dependencies

Specify public types, function/trait boundaries, protocol shapes, dependencies,
and permitted architecture edges when they are necessary to constrain the work.
Leave incidental implementation choices open.

## Recovery and idempotence

Explain which commands can be repeated safely, partial-failure cleanup, state
isolation, rollback, and any backups required for destructive work. State what
requires human authority. Never turn an uncertain operation into an implicit
permission grant.

## Evidence

Record exact commands, results, source identity, report paths, and short excerpts
that demonstrate acceptance. Separate observed results from expectations and
unrun checks. Describe manual validation precisely when automation is unsuitable.

## Outcomes and retrospective

Compare completed behavior with the original purpose. Record gaps, remaining
work, lessons, and useful harness improvements. A failed experiment can be a
valid outcome; it must not be relabeled a successful feature.

## Revision notes

Date, changed plan sections, and why. On completion move this file to completed/,
update its catalog path/status, and carry unresolved obligations into tech-debt.md.

## Task reconciliation

Name the work item and harness/tasks/ID.json record. Map acceptance to registered
requirements, account for related gaps, record material discoveries durably,
and explain documentation updates. Finalize records before CI; keep resulting
reports under target. Use partial/blocked status with next steps when needed.
