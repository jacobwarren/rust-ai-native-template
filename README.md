# Rust starter for agent-built software

A minimal Rust workspace meant to be handed to a coding agent: a small example
product with a real process boundary, one page of instructions, and one check
script. Replace the example with your product.

```sh
scripts/ci.sh                                             # the whole gate
cargo run -p starter-app -- greet Ada
cargo run -p starter-app -- capabilities
cargo run -p starter-app -- script examples/board.jsonl
```

## What is here

| Path | What it is |
| --- | --- |
| [AGENTS.md](AGENTS.md) | Everything an agent is told: three steps, the layout, a checklist for protocol changes and the rules. `CLAUDE.md` points at it. |
| `crates/domain` | Board rules and validated types. No I/O. |
| `crates/engine` | Sessions, the JSON-lines driver protocol, the journal store. |
| `crates/app` | The `starter-app` binary: a greeting and the task board's stdin and script drivers. |
| [docs/PRODUCT.md](docs/PRODUCT.md) | What the example must do. Replace it first. |
| [docs/DRIVER.md](docs/DRIVER.md) | The protocol: discover, execute, inspect, capture, replay, save, load. |
| `scripts/ci.sh` | Formatting, Clippy with warnings denied, tests, rustdoc. CI runs the same file on Linux and macOS. |
| [docs/ADOPTING.md](docs/ADOPTING.md) | Turning this into your product. |
| [docs/EVALUATION.md](docs/EVALUATION.md) | Why there is so little here, with the numbers. |

The part most worth keeping when you replace the example is the driver
pattern: the product exposes its real behavior at a process boundary an agent
can discover, drive, inspect and replay, and the tests go through it.

## Why it is this small

This repository used to carry a much larger harness: task records, transition
receipts, edit and stop hooks, a custom check runner, and up to eight reviewed
HTML pages per task. Two paired experiments on this codebase found no task
where it changed whether an agent's work was accepted. The pages cost fifteen
to forty times the wall-clock time, and the rest of the harness about three
times. So it was removed. [docs/EVALUATION.md](docs/EVALUATION.md) has the
tables and, as important, what the experiments did not test: the edit-time
hooks, and work spread over many sessions and people.

Nothing was thrown away. Three branches keep it:

- `showcase/full-harness`: the harness as it stood at the last experiment,
  with its full evaluation write-up.
- `showcase/full-ceremony`: the last commit with the eight-page set and the
  page reviewer.
- `showcase/template-history`: the template's own task history under that
  ceremony.

## Requirements

Rust from `rust-toolchain.toml` with rustfmt and Clippy, on Linux or macOS.
No project license is included; add yours before distributing.
