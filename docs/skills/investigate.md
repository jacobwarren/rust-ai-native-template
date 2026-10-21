# Repository investigation skill

Start with AGENTS.md, the product contract, and the architecture map. State the
observable behavior requested by the task and locate its entry point and callers.
Follow the data from the external boundary into domain types and back to the
public output. Inspect existing tests and compiler diagnostics before guessing
an API or adding an abstraction.

Form one specific hypothesis about where behavior must change. Use a focused
search, a minimal reproducer, or a targeted test to resolve it. Preserve a failure
case before making the fix when practical. If the same hypothesis fails twice,
review evidence and change the hypothesis instead of repeating the attempt.

Make the smallest coherent change within the permitted files and repository
boundaries. Check the requested outcome and relevant negative cases, then run
the applicable verification profile. Report observed evidence, failures, and
uncertainty. Never weaken the grader or policy, read private evaluation controls,
or claim success from a completion message or compilation alone.
