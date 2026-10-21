# Experiment boundary

Follow the [root map](../AGENTS.md) and [experiment contract](README.md).

Keep Python/model dependencies outside normal Cargo development. The evaluator
must grade observable task outcomes independently of agent claims and mutable
agent tests. Freeze task source, grader bytes, permitted paths, and agent identity.

Test no-op rejection, successful edits, protected-file edits, malformed protocols,
timeouts, and split isolation. Scripted fixtures establish plumbing only; label
them explicitly and never report their scores as model-performance improvements.

The default evaluator runs trusted processes as the same OS user. Do not claim
it hides private graders from a hostile agent. Real adversarial evaluation needs
external OS/container/service isolation and scoped credentials.

No model calls, paid experiments, or automatic skill promotion run in normal CI.
