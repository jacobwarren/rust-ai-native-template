# Reproduce, repair, retain

Use a compact regression record when a defect crosses boundaries, recurs, or
changes a safety/acceptance contract. Small fixes can use the same fields in the
PR. The template is [regressions/TEMPLATE.md](regressions/TEMPLATE.md).

1. State the user's observable failure and the exact boundary involved. Record
   the starting commit/fingerprint and known prerequisites.
2. Minimize the reproducer: request sequence, synthetic input, seed, or file.
   Write an independent expected outcome before changing the implementation.
3. Demonstrate failure on the old behavior. Preserve the command, exit, logs,
   and assertion. A compiler failure is not the desired behavioral failure.
   If the old version cannot be run, say why and mark fail-before unverified.
4. Fix the responsible boundary. Keep rejection atomic and diagnostics useful.
   Test supported neighboring behavior as well as the failing case.
5. Run the same reproducer, affected workflows, and full checks. Record the
   evidence and remaining limits. Update contracts only for intentional changes.
6. Retain the regression in normal tests/workflows; update the ranked gap and
   relevant plan. Close a gap only with a checked-in resolution record.

When you suspect a harness flaw, first demonstrate a false green or false red.
Keep a negative harness test alongside the repair. Review policy/oracle changes
explicitly and separately from implementation claims. An agent's completion
message and a review bot's approval are not independent acceptance.

An actual example from this template's development is
[0001-strict-unit-commands.md](regressions/0001-strict-unit-commands.md).
