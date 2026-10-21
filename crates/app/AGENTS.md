# Runtime boundary

Read the [root agent map](../../AGENTS.md) and [example contract](../../docs/PRODUCT.md).

Parse external input here, call the domain, and map errors to the public protocol.
Keep stdout machine-readable and stderr structured. Do not log the raw name.
Update `harness/scenarios.json` when the public contract intentionally changes.
Use per-run `APP_DATA_DIR` for local mutable state. Services should bind an
OS-assigned port and expose the selected endpoint, readiness, and shutdown.

The board's stdin driver and script adapter both use starter-engine Driver.
Keep framing bounded before parsing, preserve correlation and structured errors,
and exercise the real process boundary when changing transport. Domain and
engine tests alone do not establish transport or future UI behavior.
