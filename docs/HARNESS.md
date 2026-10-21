# Harness contract

## Execution and evidence

`cargo xtask check <profile>` loads strict JSON configuration and runs every
listed command in order, even after a check fails. It never retries a failed
check silently. Arguments are passed directly to processes, without a shell.
`@self` expands to xtask; `@app` expands to the compiled example executable.
Profiles are trusted code and must be reviewed like source.

Each step has a unique ID, a positive timeout, stdout/stderr log files, exit
code, elapsed time, and passed/failed/timeout/error status. Spawn and per-step infrastructure failures are
recorded as errors. Command specs preserve a nonempty executable, valid ID, and
positive timeout after parsing; execution statuses are typed enums. Nonzero expected application exits are evaluated by the
scenario runner, not treated as successful generic commands.

Each run gets a new directory; each command gets a separate APP_DATA_DIR.
HARNESS_RUN_ID identifies the run, HARNESS_WORKTREE_ID derives from its absolute
workspace path, and CARGO_TARGET_DIR is set to that worktree's local target
folder. Product code must actually use these settings for stateful isolation.
There is no automatic database, cache, network, or external-account isolation.
Use separate Git worktrees for concurrent source edits. Avoid concurrent runs
that edit the same checkout.

Top-level commands run in Unix process groups. Timeout cleanup kills the group;
normal completion also cleans up remaining group members. Nested harness checks
share the parent group so outer cleanup can reach them. A nested timeout kills
its immediate child; outer cleanup removes remaining ordinary descendants.
This requires Linux/macOS and /bin/kill. Descendants that deliberately daemonize
or escape the process group need stronger OS/container containment.

Before and after checks, the harness hashes relative paths and file contents in
sorted order. It excludes .git, target, .DS_Store, Python bytecode caches, and secret .env files (except
.env.example). Untracked source is included. Source symlinks are rejected.
A changed fingerprint makes the report fail. Git HEAD and dirty state are
recorded when available; a fresh extracted archive legitimately has null HEAD.

Fingerprints identify the inspected files, not tool binaries, network inputs,
environment values, or every possible runtime dependency. A running process can
edit and restore a file between hashes. Reports are local evidence, not signed
attestations or a tamper-proof audit log. The human/CI trust boundary matters.

## Profiles and contracts

- **fast:** formatting, architecture, docs, workspace unit/integration tests.
- **ci:** fast checks plus Clippy on all targets/features, rustdoc, and all six
  process scenarios. The current project has no mutually exclusive features.
- Add target/feature profiles as your product requires. If features conflict,
  replace all-features with a documented compatible feature matrix.
- Include additional integrations in the named CI profile or explicit CI jobs.
  An optional tool that is not run does not count as verification.

Reports list the checks actually performed. “Green” does not mean security,
performance, backward compatibility, or all platform behavior has been proven.
The process runner is not a credential sandbox and inherits the environment.
It captures logs to disk without a size quota; use a constrained container for
untrusted or noisy executables. Do not upload logs containing sensitive values.

## Knowledge checks

The catalog covers every Markdown file, including agent guides and templates.
Checks enforce existence, unique registration, nonempty ownership/verification,
known lifecycle status, short AGENTS maps, and local inline file links.
Supported links are ordinary `[text](relative/path.md#anchor)` syntax.
Heading anchors, reference-style links, URL availability, Markdown with nested
parentheses, ownership identity, and semantic freshness are not verified.
Use simple inline file paths; review semantic claims with code and evidence.

## Improving the harness

For a recurring failure, record the original task, failure, missing capability,
proposed signal, false-positive risk, implementation cost, and a negative test.
Promote the smallest reliable signal into a check. Do not add another page of
instructions when a compiler check or reproducible fixture can teach the rule.

For services, add JSON tracing events and request IDs first, then a real runtime
driver that starts an instance, waits for readiness, exercises flows, captures
logs/metrics/traces, and tears it down on failure. Add OpenTelemetry or a local
observability stack when a concrete question requires it. No fake dashboards or
unimplemented telemetry adapters are bundled here.

## Interrupted and failed runs

A profile checkpoints report.json before running commands and after each step.
The JSON is replaced atomically. Until all steps finish, complete=false and ok=false.
Final fingerprint failure remains an explicit error. Failure to create the run
folder or write evidence can still prevent a report; preserve process stderr.
A completed report means execution finished, not that checks passed.

## Stateful workflows and readiness

The workflow command launches the real application over stdin or file-script
transport, with shared state only between sessions of one case. Independent
expectations, strict golden comparison, request/reply logs and source identity
are recorded in workflows.json under the run directory. Missing baselines fail;
candidate files stay in artifacts. See [TESTING.md](TESTING.md).

The readiness/release commands require fresh full-CI evidence and named tests
and workflow cases. The release command fails on open scope blockers; it does
not publish. See [READINESS.md](READINESS.md). The task-board engine's limits and
persistence commit/failure semantics are in [DRIVER.md](DRIVER.md).
