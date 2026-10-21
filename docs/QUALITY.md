# Quality inventory and maintenance

Owner: maintainers. This inventory describes checked scope, not a universal grade.

| Surface | Current evidence | Gap / next trigger |
| --- | --- | --- |
| Domain examples | Greeting boundaries, board capacity, generated reference-model edit sequences | Replace with product invariants; no automatic sequence shrinking |
| Process contract | Six greeting scenarios, four stateful process workflows and transport tests | Add real workflows and UI/service drivers when adopted |
| Architecture | Every declared member and dependency edge checked | Intra-crate and std I/O reviewed manually |
| Docs | Catalog, inline file links, navigation budget | Bounded semantic review procedure; human/assigned-agent execution |
| Harness | Typed-command, report, rule, timeout, failure, and CLI tests | Extend whenever adding a guardrail |
| Portability | Linux and macOS CI jobs configured | Only the build environment is locally verified |
| Security | Unsafe-code gate and dependency boundary policy | No vulnerability scanner or threat model |
| Runtime | JSON driver/events, isolated state, atomic saves and interrupted-writer tests | Linux/macOS local filesystem contract; no service/GUI/device qualification |
| Agent effectiveness | Independent evaluator and separate final comparison | Executable trials and GEPA adapter; live model gains unmeasured |

## Maintenance loop

The weekly workflow runs the configured checks and preserves evidence. It does
not invoke a paid model, open autonomous PRs, or certify semantic doc freshness.
A maintainer or explicitly authorized agent reviews this inventory and:

1. Reproduces a specific recurring failure or identifies a concrete drift.
2. Repairs the source, contract, or check in a small change.
3. Demonstrates the old failure and new passing evidence.
4. Updates this inventory and the debt tracker if needed.

Keep optional tooling justified by failures and measured benefit. Review
suppression counts, flaky checks, diagnosis time, and repeated agent mistakes.
A dependency/toolchain upgrade must update Cargo.lock, the pin, and docs and pass
the supported CI matrix. Do not make the pinned toolchain float silently.

Known debt is recorded in [the debt tracker](plans/tech-debt.md).

Semantic upkeep follows [the maintenance procedure](MAINTENANCE.md).

Readiness is computed using [the evidence gate](READINESS.md). The ranked,
machine-readable work list is harness/gaps.json (`cargo xtask gaps`). Baselines
are reviewed source; missing baselines are failures. The CI required job collects
both the Rust matrix and optional scripted experiment contract results, but host
branch/ruleset enforcement is an adopter responsibility.
