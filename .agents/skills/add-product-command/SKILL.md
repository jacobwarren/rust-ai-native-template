---
name: add-product-command
description: Add or change a command of the example task-board engine end to end: the Command variant, its execution, the discover descriptor, saved-journal compatibility, a process workflow and the driver docs. Use when a task adds a board command.
---

# Add a product command

Use this when a task adds or changes a command of the example task board. Each
step names the file that owns it; the existing `add`, `complete`, `remove` and
`undo` commands are the pattern to follow.

1. **Domain rule first.** If the command needs a new business rule, add it to
   `crates/domain` with its own tests. The domain stays deterministic and free of
   process, storage and network concerns (`crates/domain/AGENTS.md`).
2. **The command.** Add a variant to `Command` in `crates/engine/src/lib.rs`. The
   wire format is strict: `#[serde(tag = "command", rename_all = "snake_case",
   deny_unknown_fields)]`. Execute it in `Session::execute`, which edits a cloned
   board and commits only on success, so a rejected command leaves board,
   history and journal unchanged. Map domain errors to a stable error code.
3. **Discovery.** Add a descriptor to `capabilities()` in
   `crates/engine/src/protocol.rs` with an id, a strict example payload and its
   effect. `crates/engine/tests/protocol.rs` runs every descriptor's example, so
   the example must be valid.
4. **Saved boards.** `crates/engine/src/store.rs` saves the command journal with
   `schema_version: 1` and replays it on load. A build that lacks the new variant
   cannot parse a journal that contains it. Decide whether that is acceptable or
   whether the journal format must be versioned, and record the decision in the
   task record. Versioning has been explored on the unmerged
   `trial/journal-versioning` branch; nothing on `main` supports it yet.
5. **Process evidence.** Drive the command through the real binary in a case in
   `harness/workflows.json`. Golden files under `harness/goldens/` are reviewed
   baselines: a missing golden fails, and you never regenerate one to make a
   change pass ([testing](../../../docs/TESTING.md)).
6. **Docs.** Update the surface in [the driver contract](../../../docs/DRIVER.md)
   and, if acceptance changes, [the product contract](../../../docs/PRODUCT.md).
7. **Checks.** Run `cargo xtask check fast` while editing, then
   `cargo xtask workflow`, then `cargo xtask check ci` before handoff. Changing
   `harness/workflows.json` or a golden touches protected policy, so this is a
   standard-tier task with a transition receipt
   ([task size](../../../docs/ARTIFACTS.md)).
