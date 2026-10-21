# Execution plans

Use a checked-in plan for multi-step, cross-boundary, or risky work. Start from
[the template](TEMPLATE.md), put it in `active/`, and register it in the docs
catalog. Small, reversible changes can use a brief task description instead.

A plan must stand alone: another agent should resume it without chat history.
Keep intent, acceptance checks, decisions, current evidence, and unresolved
questions accurate. On completion move it to `completed/`, update links/catalog,
set its catalog status to `archived`, and carry unresolved work into
[the debt tracker](tech-debt.md).

The completed plan records work performed on this template. Begin a new plan
for your product; keep the template's evidence separate from product claims.

## Restart contract

At a stopping point, update progress, discoveries, decisions, evidence, and the
next concrete action. On resumption, first confirm the recorded source identity
and inspect changes made since it. Re-run only checks needed to establish the
current state. Record deviations explicitly instead of repeating already verified
work or trusting a stale completion claim.

The required content is described in TEMPLATE.md. There is no minimum plan
length. A useful plan names the actual behavior, files, commands, outputs, and
recovery steps; empty headings do not make work restartable.
