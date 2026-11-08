# agent-discoverability

Written with the write-ticket skill (.claude/skills/write-ticket/SKILL.md).

## Problem
A fresh agent started at AGENTS.md and answered twelve questions about the
harness correctly, but it opened 30 files to do so and found three places where
the harness misleads it or gives no help:

- `docs/ARTIFACTS.md` "Task size" says any skill outside the ceremony "can
  change in a small task". The investigate skill also has to keep its body equal
  to its seed `docs/skills/investigate.md` (DOC009, `xtask/src/checks.rs:314`),
  so a typo fix there is a three-file edit. Only README.md says so; the agent
  found it through `harness/docs.json` and the checks source.
- `cargo xtask help` describes `agent-stop` as the "Hook gate". No hook runs it:
  `scripts/agent-complete.sh:76` runs `cargo xtask complete`, and
  `agent-stop` is used only by tests since commit 41c49e0.
- `cargo xtask task-artifact next --help` fails with `unknown command or extra
  arguments; run cargo xtask help`. There is no help for one command, and a
  command with wrong arguments does not show its usage.

## Scope
- `xtask/src/main.rs`: move the help table into one function; `--help` or `-h`
  after command words, and `help` before them, print only the matching
  entries; an unknown command or wrong arguments print the usage of the
  commands that start with the same words; `agent-stop` says what it checks
  and that the default stop hook runs `complete`. Arguments after `run --`
  stay the application's.
- `docs/ARTIFACTS.md` "Task size": the DOC009 seed rule.
- `README.md`: the command table names `help` and `--help`.
- A new test file `xtask/tests/help.rs`; its tests join the `completion`
  requirement in `harness/readiness.json`.

Tier: standard. `xtask/` and `docs/ARTIFACTS.md` are policy and ceremony paths.

## Acceptance as named tests
- `help_narrows_to_the_commands_asked`: given `cargo xtask task-artifact next
  --help`, then the reply lists exactly `task-artifact next --task ID`; given
  `cargo xtask help task-artifact`, then it lists every `task-artifact` command
  and nothing else; given `cargo xtask nonsense --help`, then it fails and
  points at `cargo xtask help`.
- `wrong_arguments_print_the_matching_usage`: given `cargo xtask complete
  --task X`, then the error contains `complete --task ID --base REV --report
  PATH`; given an unknown first word, then the error points at `cargo xtask
  help` and lists no usage.
- `help_flags_after_run_belong_to_the_application`: given `cargo xtask run --
  --help`, then xtask does not print its own help.
- `agent_stop_help_names_the_hook_that_runs`: given `cargo xtask agent-stop
  --help`, then the entry says the default stop hook runs `complete`, and
  `scripts/agent-complete.sh` runs `complete`, not `agent-stop`.

## Validation
`cargo xtask check ci > target/verification/ci.json`;
`cargo xtask gaps --base origin/main` lists the protected changes for the
receipt;
`cargo xtask complete --task agent-discoverability --base origin/main --report target/verification/ci.json`.

## Out of scope
- Removing `agent-stop`: it is tested and still gives one task's stop verdict
  by hand.
- Precedence between `.claude/settings.local.json` allows and the project's
  `ask` rules; that file is per machine and ignored.
- The FINDINGS wording difference between `docs/ARTIFACTS.md` and the
  review-artifact skill, which harness/tasks/next-prompt.json already records.
- Rewriting AGENTS.md or reducing how many files an agent opens.
