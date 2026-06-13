# Scripts

Purpose: thin command entrypoints for build, verification, generation, migration, packaging, and release workflows.

Owner: SDKWork Knowledgebase maintainers.

Allowed content: thin wrappers that delegate reusable logic to `tools/` or package/crate commands.

Forbidden content: reusable parsers/generators/validators, long-lived business logic, generated SDK output, runtime state, and secrets.

Related specs: `../sdkwork-specs/SDKWORK_WORKSPACE_SPEC.md`, `../sdkwork-specs/ENGINEERING_WORKFLOW_SPEC.md`.

Verification: `powershell -ExecutionPolicy Bypass -File tools/verify_sdkwork_structure.ps1`.

Status: inactive placeholder. Current reusable verification logic lives under `tools/`.
