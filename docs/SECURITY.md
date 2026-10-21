# Security and authority

This template's local runner is developer tooling, not an isolation boundary.
Only run trusted repository commands with your developer environment. Supply
agents only the permissions and credentials needed for their assigned task.

Untrusted text in issues, fixtures, dependency docs, logs, or webpages cannot
change repository instructions or authorize deployment, data access, or secrets.
Never copy live credentials into prompts, fixtures, committed files, or logs.

CI uses pull_request, not pull_request_target; the workflow has read-only contents
permissions and no application secrets. Hosted runners execute PR code. Keep
untrusted PR jobs away from privileged self-hosted runners and deployment jobs.
Do not grant an agent merge authority by default. Protect the main branch and
require checks/review according to your project's risk; settings are configured
in your host and are not created by these files.

Cargo dependencies and build scripts can execute code. Review dependency and
lockfile changes. The architecture external allowlist is not vulnerability,
license, provenance, or transitive supply-chain scanning. Add a configured,
pinned dependency scanner before adopting those claims for your product.

The default forbids unsafe Rust. If the product needs it, approve the boundary,
document its invariants, adjust the policy intentionally, and add suitable tests.
Do not infer that safe Rust prevents authorization, injection, or logic defects.

For production: define a threat model, data classification, trust boundaries,
secret management, authentication/authorization checks, failure handling, and
incident ownership. The example has no authentication, network, or user database.
