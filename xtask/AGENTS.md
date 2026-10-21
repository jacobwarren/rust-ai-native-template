# Harness implementation

Read the [root agent map](../AGENTS.md) and [harness contract](../docs/HARNESS.md).

Changes here affect the credibility of every application check. Exercise both
passing and failing paths, including malformed input, forbidden dependencies,
process failure, timeout, and report status. Preserve machine-readable output.

Keep config parsing strict. Never silently skip unknown members or empty suites.
Do not add model SDKs, autonomous execution, credentials, or network services
as prerequisites for normal development. The runner executes commands supplied
by trusted repository configuration; it is not a sandbox.
