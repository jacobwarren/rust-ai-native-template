# Engineering principles

- Keep intent, contracts, and consequential decisions in versioned files.
- Prefer typed boundaries, explicit errors, deterministic core behavior, and
  boring dependencies that agents can inspect. Avoid unchecked input probes.
- Follow existing shapes unless there is evidence for a better one. Duplicate
  logic is a reason to inspect responsibilities, not immediately create a framework.
- Test meaningful boundaries: malformed input, ownership, failure, persistence,
  cancellation, protocol compatibility, and invariants applicable to the product.
- Preserve a regression reproducer before fixing a bug when practical. A test
  that restates implementation details provides weak evidence.
- Never use a green test suite to dismiss missing acceptance criteria.
- Keep changes reviewable. Explain what behavior changed, why, evidence, and
  remaining uncertainty. Update affected docs in the same change.
- Permit local implementation freedom inside enforced architecture. Adding a
  dependency, unsafe code, a new layer, or a weakened gate deserves an explicit
  decision. Do not cargo-cult the example into an unsuitable product.
- Rust-specific tools are optional until relevant: cargo-deny/audit for dependency
  policy, cargo-nextest for test execution scale, cargo-llvm-cov for coverage
  questions, cargo-mutants for assertion strength, cargo-fuzz for parsers, Miri
  for unsafe-sensitive paths. Pin chosen versions and add actual CI execution.
  None is installed or claimed as a passed check by this template.

The outer agent decides how to investigate; this repository supplies context,
commands, constraints, and evidence. It does not implement its own LLM scheduler.
