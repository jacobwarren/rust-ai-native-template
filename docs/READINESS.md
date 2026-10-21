# Evidence-backed readiness

Feature presence, verified behavior, and release readiness are distinct.

- implemented: registered implementation files exist; this is inventory only.
- verified: current full-CI evidence includes the named passing tests and public
  workflows for the requirement.
- release_ready: every requirement in a named scope is verified and that scope
  has no open blocking gaps.

harness/readiness.json registers requirements and scopes. harness/gaps.json is the
ranked work list with impact, evidence, next action and blocking scopes. A closed
gap needs a checked-in resolution document. Reviewers judge that resolution;
file existence alone cannot prove its claims. Pick the highest-impact applicable
open gap instead of inventing feature breadth work.

```sh
cargo xtask gaps
cargo xtask readiness
mkdir -p target/verification
cargo xtask check ci > target/verification/ci.json
cargo xtask readiness --report target/verification/ci.json
cargo xtask release template --report target/verification/ci.json
cargo xtask release product --report target/verification/ci.json
```

The last command deliberately fails in the unadopted template. Product acceptance,
platform qualification and hosted enforcement are open blockers. The template
scope qualifies the included examples and harness only. It does not certify an
arbitrary product, remote CI execution, macOS behavior based on Linux logs, or
security. A successful readiness inspection command can report blocked scopes;
use release for a nonzero gating exit. Neither command publishes or deploys.

The verifier rejects stale/failed/incomplete CI reports, source changes, a check
set or argv that differs from current CI policy, missing named tests, and missing
or failed process workflows. Changing source, requirements or gaps invalidates
previous reports; rerun CI. It reads the referenced test/workflow logs, so retain
them with the report. Logs and reports are trusted local evidence, not signed
attestations. Hosting permissions and protected review provide authority.

## Hosting

The PR workflow runs the full Rust gate on Linux and macOS and the separate
scripted experiment contracts. Its final required job fails if either dependency
fails, is cancelled, or is skipped. Make required a required status check in the
adopted repository's ruleset/branch protection. Require independent review for
harness policy, workflows, baselines, readiness and gap-resolution changes; assign
real owners using CODEOWNERS when the owners are known. Do not add fictitious
usernames to the starter. Restrict bypass rights according to your team policy.

No remote repository was supplied with this archive, so branch protection and
remote CI qualification must be configured and observed after adoption. Keep the
hosting-enforcement gap open until that has happened.
