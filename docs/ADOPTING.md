# Adopt this template

1. Run `scripts/ci.sh` before editing, so you know the baseline is green.
2. Replace [PRODUCT.md](PRODUCT.md) with your product: who uses it, the
   observable outcomes, what it must reject, the platforms it supports. Attach
   each outcome to a test or an explicitly manual check.
3. Choose the smallest crate layout that fits. Rename the crates and the
   binary, or replace the example outright. Run `cargo generate-lockfile` after
   intentional dependency changes and review the lockfile diff; `scripts/ci.sh`
   builds with `--locked`.
4. Keep the shape of the example where it applies: validated types at the
   edge of the domain, one command path, bounded inputs, and a driver that
   exposes real behavior at a process boundary. [DRIVER.md](DRIVER.md) is an
   example of that contract, not a universal interface. For a service, start
   on an OS-assigned port, print the port when ready, and have the tests speak
   to the running process.
5. Rewrite the checklist in [AGENTS.md](../AGENTS.md) for your product. It
   holds the things agents left out when the request did not spell them out;
   yours will be different, and you find them by grading work, not by
   guessing.
6. Enable GitHub Actions and make the `ci` job a required check. The file
   alone does not protect a branch.

## A first prompt

```text
Read AGENTS.md. We are turning this starter into: [product and platform].
The first useful slice is: [observable outcome].
It must accept: [valid behavior]. It must reject: [invalid behavior].
Constraints: [dependencies, runtime, security, compatibility].
Replace the example where it is in the way, keep scripts/ci.sh passing,
and tell me what you verified and what you did not.
```

## Adding process

Add a rule, a hook or a document only when graded work shows agents getting
something wrong without it, and check afterwards that it helped.
[EVALUATION.md](EVALUATION.md) describes how this repository measured that and
what it removed as a result. The removed harness is on the
`showcase/full-harness` branch if you want any part of it back.
