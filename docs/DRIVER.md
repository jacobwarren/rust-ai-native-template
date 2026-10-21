# Application driver contract

The task board is a disposable stateful reference. The application exposes its
real behavior through starter-engine Session; both transport adapters use Driver.
Normal use requires no model, Python, sockets, or MCP server.

## Discover and exercise

From the workspace root:

```sh
cargo xtask run -- capabilities
cargo xtask run -- script examples/board.jsonl
cargo xtask workflow
cargo xtask workflow board-lifecycle
```

The harness creates isolated state, captures stdout/stderr and reports its paths.
For interactive integration, build once and start target/debug/starter-app driver.
Each UTF-8 JSON line is one request; EOF ends the session. For persistent manual
use, set APP_DATA_DIR to your own private directory. The store acquires a
nonblocking exclusive OS lock for the process lifetime; another writer fails.
A new process starts empty; request load explicitly to restore the saved board.

```json
{"schema_version":1,"id":1,"action":{"method":"discover"}}
{"schema_version":1,"id":2,"action":{"method":"execute","command":{"command":"add","title":"Review"}}}
{"schema_version":1,"id":3,"action":{"method":"inspect"}}
{"schema_version":1,"id":4,"action":{"method":"save"}}
{"schema_version":1,"id":5,"action":{"method":"capture"}}
```

Every response has schema_version, id, ok, and either result or error with code
and message. An unparseable request has id=null. Stdout is protocol data. Stderr
contains driver_response events with run_id and ok, never titles or full input.
Process exit is 0 when every request succeeds, 2 if any request fails. Complete
malformed frames receive an error and processing continues. A frame exceeding
256 KiB including its newline receives an error and closes input processing;
the tail is never interpreted as a new command. Output is flushed after each reply.

## Surface

| Method | Inputs | Result and boundary |
| --- | --- | --- |
| discover | none | Version, limits, methods and typed command examples |
| execute | command | One edit through Session; domain rejection preserves all state |
| inspect | none | Ordered tasks, next_id, revision, can_undo |
| capture | none | Snapshot, journal, snapshot_sha256 and application_engine boundary |
| replay | commands | Rebuild from an empty session; commit only if every command succeeds |
| save | none | Persist journal to APP_DATA_DIR/board.json |
| load | none | Validate stored version/checksum, replay, then replace live state |

Commands are add {title}, complete {id}, remove {id}, undo {} with the discriminator
command. Unknown fields at every request/command level fail admission, including
empty-struct variants. IDs are unsigned integers. Title parsing rejects controls
before trimming, requires 1–120 Unicode scalar values after trimming, and caps
raw input at 512 bytes. A board holds at most 64 tasks; a session journal holds
at most 256 successful commands, including undo. Undo restores the prior board
and next_id; IDs can therefore be reused after undo. Revision counts successful
journal entries. No command retry deduplication is implied: replay replaces the
session, while repeating add creates another task.

Capture is data returned to the caller, not an arbitrary filesystem-write tool.
It includes user data. Review/redact artifacts before sharing them. The checksum
identifies content and detects accidental corruption; it is not authentication.

## Persistence and recovery

Store writes a bounded versioned journal and checksum to a new file in the same
directory, syncs it, atomically renames it over board.json, then syncs the directory.
Linux/macOS local filesystems are the supported scope. This does not promise
correctness on every network filesystem or physical survival of every power loss.

An error before replacement leaves the old file intact. If directory sync fails
after replacement, save_uncertain explicitly says the new file is visible but
durability is unconfirmed. Reopen/inspect before deciding whether to retry. Live
state is never rolled back merely because save failed. Unsaved edits remain
unsaved and are lost on process exit.

After an interrupted write, load reads only board.json. Orphan .board-*.tmp files
are ignored, never promoted automatically. They may be removed while no writer
is running. A missing or corrupt primary file is an explicit error; the driver
never silently replaces it with empty state. Loading bad data preserves the live
session. Tests inject errors before/after replacement and exit a child process
after syncing its staged file, then reopen the last committed state.

APP_DATA_DIR must be private to the operator. The advisory writer lock coordinates
cooperating instances, not hostile filesystem writers. There is no path sandbox,
remote authentication, automatic backup archive, or recovery of unsaved work.

## Adapters for a real product

Keep a small capability-discovery and state-inspection surface beside the real
public execution path. A service can expose a local test driver and real HTTP
requests; a library can use an external consumer; a GUI needs real input and
screenshot capture as well as engine tests. Describe what each test reaches.
Add MCP only when a consumer needs it, as an adapter over the same operations.
No graphical, browser, network, or hardware behavior is demonstrated by this CLI.
