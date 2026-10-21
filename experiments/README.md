# Executable agent-task evaluation

This optional subsystem answers whether a coding agent completed a requested task.
It uses Python 3.11+ standard library, Git, and the task project's toolchain on
Linux/macOS. Cargo development itself has no Python dependency.

## Run the real Rust plumbing demonstration

From the starter root, choose a new output directory:

```sh
PYTHONDONTWRITEBYTECODE=1 python3 -m unittest discover -s experiments/tests -v
python3 -m experiments.evaluator.demo --output target/experiments/my-demo
```

The demo creates a disposable Rust Git repository and an independent Unicode
uppercase-output grader. A scripted no-op agent must be rejected even though its
regression tests pass; a scripted correct patch must be accepted; an edit to the
protected Cargo.toml must be a policy violation. This verifies the evaluator, not
a model's ability. Reusing the same demo output path is rejected to preserve evidence.

## Prepare an actual task suite

Copy [the suite example](suite.example.json) to a control directory outside the
task repository. Fill in absolute paths and a full immutable Git commit hash.
Each task needs a real user prompt, exact permitted source files, at least one
regression command, and a separate Python grader script. The grader must fail on
the baseline and pass only when the requested behavior is observed.

The baseline must pass the regression commands. Otherwise the evaluator marks
the task invalid rather than teaching the optimizer to solve a broken environment.
The input working tree is never used as the baseline: only the named commit is
exported, so commit intentional task setup before pinning it. Uncommitted changes
remain untouched. Symlinks, special files, and submodules are unsupported.

```sh
python3 -m experiments.evaluator --suite /absolute/control/train.json --candidate docs/skills/investigate.md --output /absolute/results/baseline --repeats 3
```

Suite parsing freezes commands and grader bytes before running an agent. A trial:

1. Exports the pinned commit, checks baseline acceptance fails and regressions pass.
2. Creates a fresh Git-initialized snapshot for the agent (no original history).
3. Sends a task request on stdin to the configured agent command.
4. Inventories source files independently of the agent's Git index.
5. Rejects any source change outside the exact allowed-file list.
6. Recreates a clean grading tree and imports only the permitted patch, discarding
   agent build output and Git metadata.
7. Runs fixed regressions and the frozen external acceptance grader. Both must pass.

Only root .git and target are excluded as agent runtime directories. Other new
files require explicit permission. Source-content and executable-bit changes
are recorded. Graders and regression commands may write target artifacts but
must not mutate the source tree. Log files and changed-file evidence remain;
large temporary checkouts are removed after the trial.

## Agent protocol

The command receives one JSON object on stdin with schema_version=1, task_id,
workspace, prompt, candidate_skill, and editable_files. Its current directory is
the agent workspace. It must exit zero and emit one JSON response on stdout:

```json
{"schema_version":1,"claimed_complete":true,"summary":"What changed", "usage":{"input_tokens":null,"output_tokens":null,"cost_usd":null}}
```

Usage is optional and explicitly agent-reported. Missing usage and human effort
remain null; they are never treated as zero cost. Agent logs belong on stderr.
A malformed response is a protocol error. Completion claims are evidence only,
never the grade. Exit failure, timeout, protocol failure, and rejected task are
separate report statuses.

A bridge for an existing CLI that reads a prompt from stdin is provided:
`experiments/evaluator/stdin_agent.py -- <agent executable> <arguments>`.
Use its absolute script path in the suite's agent.argv. Configure your installed
agent's noninteractive stdin mode explicitly. The bridge forwards the prompt,
logs output, and reports unknown usage; it does not invent provider-specific
flags, authentication, token accounting, or completion detection.

## Grader protocol

The grader runs with the clean grading workspace as cwd. It receives no candidate
skill and no agent response. Use subprocess calls, an external consumer, API
requests, or other checks of the actual requested outcome. Stdout is exactly:

```json
{"accepted":false,"reason":"Expected output or invariant was not observed"}
```

Exit 0 when accepted=true; exit 1 when accepted=false. Diagnostics go to stderr.
Malformed output, inconsistent exit codes, or timeouts are evaluator errors. The
[demo grader](evaluator/demo.py) supplies an executable example. Python scripts can
invoke arbitrary Rust tests/binaries; this is a protocol, not a restriction to
Python product implementations. Multi-file grader bundles need an extension.

## Evidence and limits

Each trial checkpoints report.json before execution and after the agent. Normal
completion includes exact commit, suite/grader/candidate digests, source digests,
changes, commands, statuses, durations, logs, and independent results. An abrupt
interrupt leaves complete=false. Insufficient disk space can prevent reporting.

This is controlled evaluation of trusted local agents, not a security boundary.
Agent and grader processes share the OS user and can access host paths if they
try; hiding paths in the request does not prevent discovery. Frozen controls and
patch reconstruction prevent accidental grading drift, not malicious host access.
Use separate containers/users or a grading service for genuinely private holdouts,
network restrictions, and hostile generated code. The runner inherits environment
credentials and has no disk quota. Root target contents are discarded for grading.

Command timeouts cover ordinary Unix process groups. The current evaluator does
not trace every agent tool call or independently meter tokens/spend. SDK/API usage
requires an accounting-aware agent adapter. Do not substitute tool duration for
model compute cost. Repeated trials expose variance; review per-task regressions.

See [GEPA setup](gepa/README.md) for optional skill search and holdout comparison.
