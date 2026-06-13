# Tests

Purpose: cross-package tests, contract tests, integration tests, end-to-end tests, fixtures, and static verification inputs.

Owner: SDKWork Knowledgebase maintainers.

Allowed content: repository-level tests and safe fixtures that span package boundaries.

Forbidden content: package-local unit tests that belong beside their crate, real secrets, tokens, private customer data, runtime state, and generated SDK output.

Related specs: `../sdkwork-specs/TEST_SPEC.md`, `../sdkwork-specs/SDKWORK_WORKSPACE_SPEC.md`.

Verification: `powershell -ExecutionPolicy Bypass -File tools/verify_sdkwork_structure.ps1`.

Status: inactive placeholder. Current Rust tests remain package-local under `crates/*/tests/`.
