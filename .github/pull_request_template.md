## Why

Problem and observable outcome.

## Change

Behavior, boundaries, contracts, and intentional policy changes.

## Evidence

Exact commands, report paths, affected acceptance criteria, and source identity.
Include pre-existing failures and checks not run. Link the plan when applicable.

## Risk and uncertainty

Compatibility, security, rollback, and remaining gaps. No blanket “all safe”.

## Behavioral regression and readiness

Which user/runtime boundary was exercised? Link the minimized reproducer and
fail-before/pass-after evidence, or explain why fail-before is unverified.
Identify changed golden baselines, harness policies, or acceptance criteria for
explicit review. State the qualified release scope and open ranked blockers.

## Task reconciliation

Link harness/tasks/ID.json and its work item. Account for related gaps and new
discoveries, or state none discovered. Link the handoff report and any transition
receipts. Distinguish complete acceptance from partial/blocked handoff. Review
protected policy/baseline changes independently; a receipt is rationale, not approval.

State the inspected scope and review of acceptance, check failures, assumptions
and missing coverage. Distinguish durable discoveries from transient failures
already fixed. No new gap is required when none was discovered.
