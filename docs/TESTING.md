# Behavioral evidence and baselines

Use the shortest check that reaches the behavior under investigation, then the
full pre-merge gate. All commands below run from the workspace root.

| Evidence | Command / location | What it establishes |
| --- | --- | --- |
| Domain boundaries | cargo test -p starter-domain | Validation, capacity and state invariants |
| Generated sequences | cargo test -p starter-engine --test properties | 64 fixed seeds × 100 edits against an independent Vec model; replay equivalence |
| Failure recovery | cargo test -p starter-engine store::tests | Save/reopen, locks, corruption, injected I/O failures, interrupted writer |
| Transport | cargo test -p starter-app --test driver_boundary | Actual process framing, oversized-input closure, script/stdin equivalence |
| Workflows | cargo xtask workflow | Complete process journeys with independent expectations |
| Full gate | cargo xtask check ci | Formatting, dependencies, docs, Clippy, tests, rustdoc, greeting scenarios, board workflows |

Generated tests are deterministic and bounded, with seed/operation diagnostics.
They do not automatically shrink failures; minimize a failing sequence and retain
it as a regression. They supplement examples and boundary tests. Do not count
6,400 generated operations as 6,400 independent tests or as production coverage.

## Workflow evidence

harness/workflows.json declares criterion, boundary, transport, requests, expected
exit and one expectation per response. Sessions within a case share one isolated
state directory, so reopening uses a new real process. Cases never share state.
The runner uses bounded processes and file-backed input/output. It records input,
reply, telemetry, comparisons and source fingerprint under target/harness/runs/.

Exact expectations compare the complete JSON response. Fields expectations compare
named JSON pointers and must specify /ok; use them when diagnostic text is not a
stable contract. Golden expectations require a successful response and compare
the specified state against a reviewed file. These are explicit assertions, not
a product-generated score. No GUI or operating-system event is simulated by an
engine-level check; declare the actual boundary when adding another driver.

## Golden integrity

harness/goldens contains authored expected task-board states. A missing, invalid,
or mismatched baseline fails. Verification never writes the source baseline.
A failed comparison writes a candidate JSON file only under the run's artifacts.

1. Reproduce the failure with cargo xtask workflow <id>.
2. Read the actual criterion, request trace, and baseline. Decide whether the
   behavior is wrong or the contract intentionally changed.
3. Compare the artifact candidate with the checked-in baseline using diff -u.
4. If the change is intended, edit the baseline explicitly and review it together
   with the changed requirement. Require independent review for meaningful oracle
   changes. Never regenerate expectations just to make an agent's patch pass.
5. Run the workflow and full checks again; source identity has changed.

No automatic bless switch exists. For graphical products, add image comparisons
with justified tolerances and review actual screenshots; JSON snapshots alone
cannot establish visual quality. Missing external corpora must fail required
acceptance gates or be reported as unverified, never silently green.

## Turning defects into permanent checks

Follow [the regression procedure](REGRESSIONS.md). Maintain both allowed and
rejected cases when strengthening a boundary. Add structural checks only for a
specific architectural rule; retain benign cases to catch over-broad enforcement.
Source fingerprints detect check-time edits; trusted CI and review govern changes
to the harness and expectations themselves.
