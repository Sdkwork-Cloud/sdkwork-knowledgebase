# Plugins

Purpose: application/runtime plugin source packages.

Owner: SDKWork Knowledgebase maintainers.

Allowed content: SDKWork Knowledgebase runtime plugin source, plugin specs, tests, and documentation.

Forbidden content: repository/application agent plugins, generated SDK output, runtime databases, logs, cache, secrets, and unrelated vendored toolchains.

Related specs: `../sdkwork-specs/SDKWORK_WORKSPACE_SPEC.md`, `../sdkwork-specs/COMPONENT_SPEC.md`.

Verification: `powershell -ExecutionPolicy Bypass -File tools/verify_sdkwork_structure.ps1`.

Status: inactive placeholder. Agent plugins belong under `.sdkwork/plugins/`.
