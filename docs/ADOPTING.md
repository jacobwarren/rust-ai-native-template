# Adopt this template

## First hour

1. Run `cargo xtask doctor` and `cargo xtask check ci` before editing.
2. Replace PRODUCT.md with your actual purpose, acceptance criteria, non-goals,
   supported platforms, runtime shape, and failure/resource budgets.
3. Choose the smallest crate layout that fits. Keep or replace the example;
   do not add six layers before you need them.
4. Rename crates and update Cargo dependencies, package metadata, harness
   config, the @app path/build package in xtask, scenario contracts, and docs
   together. Run `cargo generate-lockfile` after intentional dependency changes.
   Review the lockfile diff, then return to locked verification.
5. Replace “maintainers” with actual ownership in metadata and the docs catalog.
   Add your license before public distribution; no project license is assumed.
6. Keep meaningful behavior tests. Replace the disposable greeting scenarios
   and board workflows with real user journeys and negative inputs for your product.
7. Run full checks and inspect the JSON report. Commit the clean baseline.
8. Enable GitHub Actions and required status checks/review in repository settings.
   The files alone do not enable branch protection, publishing, or deployment.

## First-task prompt

```text
Read AGENTS.md and the documents relevant to this task.
We are adapting this starter into: [product and target platform].
The first useful vertical slice is: [observable outcome].
Acceptance criteria: [specific valid and invalid behavior].
Constraints: [dependencies, runtime, security, latency, compatibility].
Inspect the baseline and propose the smallest crate/contract changes needed.
Implement the slice, replace the greeting sample where appropriate, update
architecture policy and docs intentionally, run the harness, and report exact
verification evidence and unresolved limitations. Do not claim unrun checks.
```

For complex work, create an execution plan. For small changes, act directly.
An agent may review its own diff; consequential policy/authority changes benefit
from an independent reviewer. Neither self-review nor a second model guarantees
correctness, so preserve independent acceptance checks.

## Worktrees

```sh
git worktree add ../my-product-feature -b feature/example
cd ../my-product-feature
cargo xtask check fast
```

The harness uses this checkout's target folder and per-command state paths.
For external services, databases, caches, or hardware, add real isolation in the
runtime driver. For an HTTP server, bind port zero and report the assigned port;
path hashes do not guarantee collision-free network ports.

## Extending platform coverage

Linux and macOS are configured. Windows process-tree containment is not
implemented; port it with job objects or another verified mechanism before
claiming Windows harness support. Embedded/no_std products need a compatible
host harness plus target-specific build/run checks. Toolchain/MSRV claims require
actual tested versions; the pin is not evidence for older compilers.

## Establish the product's acceptance surface

Expose discovery, execution, inspection and evidence capture at the real runtime
boundary; reuse the same product implementation for user and agent adapters.
Follow [the driver contract](DRIVER.md) as an example, not a universal interface.
Replace harness/workflows.json and harness/goldens with reviewed product outcomes.
Keep property/replay/failure-atomicity tests where those invariants apply.

Register actual requirements in harness/readiness.json and close the ranked
harness/gaps.json blockers with checked-in resolution evidence. Keep template and
product claims separate. Set the workflow's required job as a required status and
assign real reviewers to policies, baselines, workflows and gap closure. The
archive cannot configure your hosting settings. See [READINESS.md](READINESS.md).

## Work tracking and agent completion

Keep task coordination wherever your team works. The task record may contain an
opaque work_item reference; prompts need no ticket. Keep only material engineering
shortcomings in the project gap register. Use the task-review checkpoint and
optional stop hook before completion claims. See [COMPLETION.md](COMPLETION.md).
Live tracker synchronization is an optional team integration outside the core.
