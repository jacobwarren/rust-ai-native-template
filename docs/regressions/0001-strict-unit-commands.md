# Reject unknown fields on parameterless commands

Status: repaired. Owner: maintainers. Introduced while implementing plan 0003.

## Boundary and expectation

The public JSON-lines driver must reject fields not declared by the command.
A command object with command=undo and typo=true must fail without undoing an
edit. The same applies to an inspect action with an unexpected field.

## Fail-before

The first engine integration run failed
strict_requests_and_nested_commands_fail_without_mutation: expected ok=false,
observed ok=true. Unit enum variants were accepted with extra fields even with
serde's deny_unknown_fields annotation. The development log is included as
regression-before.log in the delivered verification evidence. This was an observed
behavioral failure, not an inferred regression or a compiler error.

## Repair

Parameterless variants are empty struct variants in Command and Action, so the
strict deserializer validates their empty field set. Engine admission tests cover
nested command and outer action fields; the rejected-edit process workflow checks
that the malformed undo request leaves the saved snapshot unchanged.

## Pass-after and scope

cargo test -p starter-engine --test protocol and cargo xtask workflow rejected-edit
pass. The final verification index provides the full source-bound report. This
establishes this protocol boundary; it is not a general guarantee about every
Serde representation. Keep the process test when changing the schema.
