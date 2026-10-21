# Optional GEPA skill optimization

The candidate is one repository-investigation skill. The evaluator, regression
commands, permitted edit paths, agent identity, and graders remain fixed.
The initial seed lives in docs/skills/investigate.md. No changes are automatically
applied to that file by search or final evaluation.

## Install the tested API in an isolated environment

```sh
python3 -m venv target/gepa-venv
target/gepa-venv/bin/pip install -r experiments/gepa/requirements.lock
PYTHONDONTWRITEBYTECODE=1 target/gepa-venv/bin/python -m unittest experiments.tests.test_gepa -v
```

The lock file pins the installed Python 3.12/Linux dependency set, including
GEPA 0.1.4 and LiteLLM. It is a version lock, not a wheel/hash lock or proof for
all Python/platform combinations. The API adapter checks the GEPA version.
Tests exercise the real package with a scripted proposer and fixture coding agent;
no API key or model call is required. GEPA-only tests are explicitly skipped when
the optional environment is absent.

Important version detail: GEPA 0.1.4 exposes GEPAConfig, EngineConfig, and
ReflectionConfig in gepa.optimize_anything. Latest upstream documentation may
show OptimizeAnythingConfig and test_set. This adapter uses the installed,
verified API and performs holdout scoring as a separate explicit command.
Do not copy a newer API example into this pinned adapter without testing it.

## Configure search

Create actual training and validation task suites using the evaluator protocol.
Keep validation tasks out of the training split and use the same coding agent,
model/version, command, and limits. [The experiment example](experiment.example.json)
shows all required fields; paths resolve relative to that configuration file.
The provided Rust demo is a plumbing fixture, not a sufficient optimization dataset.

Choose the reflection_model explicitly and configure its provider credentials in
the process environment. The coding agent's configuration and credentials are
separate. No credentials are bundled and no live model run has been performed.

```sh
target/gepa-venv/bin/python -m experiments.gepa --config /absolute/control/experiment.json --output /absolute/results/search-001
```

This explicit command may spend money on both reflection and coding-agent calls.
Search freezes the loaded suites and grader bytes. It passes only training and
validation task IDs to GEPA. Duplicate task IDs or matching commit/allowed-files/
grader fingerprints across splits are rejected, even after renaming or rewording
the task. Semantic near-duplicates still require dataset review.

The primary score is independently accepted task fraction. Time, failures, and
usage are recorded separately; faster wrong changes cannot offset failed tasks.
Feedback includes acceptance diagnostics and regression statuses. Raw agent logs
are retained locally and are not automatically sent to the reflection model.
Infrastructure failures and malformed agent protocols abort search with an
incomplete report; they are not optimized as ordinary task failures.

## Budget semantics

max_evals is enforced at trial admission in addition to GEPA's metric-call limit.
max_proposals bounds search proposals. admission_seconds stops admitting trials
after the elapsed deadline; it does not interrupt an already admitted trial.
Agent, grader, and regression commands each have their own process timeout.
Reflection calls use a 60-second request timeout and no configured retries.
A hard admission stop aborts search and leaves complete=false instead of selecting
from a partly evaluated generation. Use a larger deliberate budget to rerun.

max_reflection_cost_usd limits GEPA-reported proposer spend, not coding-agent
spend. It is checked by GEPA between calls and may overshoot by an in-flight call.
Missing coding-agent usage stays null. Set provider/agent-side spend limits for
hard financial control. There is no claim of a hard combined dollar or wall-time
limit, or of perfect containment of daemonized descendants.

## Review the selection, then evaluate holdouts explicitly

Search creates optimization.json, seed-skill.md, selected-skill.md, and selection.json,
plus per-trial evidence and GEPA artifacts. Selection is based on validation data,
which is therefore not a final untouched test set. Read the optimization log for
reflection errors or a search that made no useful proposal.

```sh
target/gepa-venv/bin/python -m experiments.gepa.finalize --selection /absolute/results/search-001/selection.json --suite /absolute/private/holdout.json --output /absolute/results/final-001 --repeats 3 --max-trials 100
```

Holdouts never enter the search function. Final evaluation checks skill hashes,
agent identity/command/timeout, split overlap, and a total trial-count cap. It runs
seed and selected skills on the same tasks with alternating order. comparison.json
records acceptance, elapsed times, per-task outcomes, and any reported usage.
After observing a holdout, treat it as consumed; do not tune on it and still call
it untouched. Same-user filesystem access is not a privacy boundary: use external
isolation when hidden graders must resist adversarial access.

Promotion is always a reviewed diff. Set minimum benefit, tolerated regressions,
and time/cost constraints before the experiment. Inspect per-task failures and
repeat variance, and keep the seed if evidence is inconclusive. No one successful
run demonstrates gains across arbitrary Rust repositories or different agents.
