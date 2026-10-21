# Domain boundary

Read the [root agent map](../../AGENTS.md).

Keep business rules deterministic and independent of process, storage, and
network concerns. Inputs become validated domain types at the boundary.
Return explicit errors. No external dependencies are allowed by default.

Cargo's dependency graph is checked, but this does not forbid `std::fs`,
`std::net`, or interior module coupling: review those properties explicitly.
As the domain grows, split by business capability only when that improves
ownership, dependency enforcement, or independent testing.
