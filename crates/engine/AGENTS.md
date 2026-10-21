# Application engine

Read the [root map](../../AGENTS.md) and [driver contract](../../docs/DRIVER.md).

Session is the one command path for all adapters. Reject invalid input before
mutation; replay/load commit a complete valid session or preserve the old one.
Keep command, journal, and input sizes bounded. Store owns filesystem effects.
Never hide a save error or claim rollback after replacement already occurred.

Exercise independent expected state, replay, undo, failed persistence, and actual
process interruption. Fault hooks stay inside test builds; no production request
can trigger them. Golden files require explicit review and never auto-create.
