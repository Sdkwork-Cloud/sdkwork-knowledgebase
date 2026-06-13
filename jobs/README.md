# Jobs

Purpose: schedules, queue bindings, batch descriptors, and maintenance runbooks.

Owner: SDKWork Knowledgebase maintainers.

Allowed content: job definitions, schedule metadata, queue contracts, batch descriptors, and operational runbooks.

Forbidden content: Rust worker implementation code, generated SDK output, runtime state, secrets, and local execution logs.

Related specs: `../sdkwork-specs/SDKWORK_WORKSPACE_SPEC.md`, `../sdkwork-specs/RUST_CODE_SPEC.md`.

Verification: `powershell -ExecutionPolicy Bypass -File tools/verify_sdkwork_structure.ps1`.

Status: inactive placeholder. Rust worker implementations, if added later, belong under `crates/sdkwork-intelligence-<capability>-worker/`.
