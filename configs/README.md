# Configs

Purpose: source-controlled safe config templates, schemas, profile examples, and non-secret defaults.

Owner: SDKWork Knowledgebase maintainers.

Allowed content: checked-in config schemas, examples, and non-secret defaults.

Forbidden content: `.local` overrides, live credentials, access tokens, private keys, runtime user config, database files, and Redis data.

Related specs: `../sdkwork-specs/SDKWORK_WORKSPACE_SPEC.md`, `../sdkwork-specs/CONFIG_SPEC.md`, `../sdkwork-specs/ENVIRONMENT_SPEC.md`, `../sdkwork-specs/RUNTIME_DIRECTORY_SPEC.md`.

Verification: `powershell -ExecutionPolicy Bypass -File tools/verify_sdkwork_structure.ps1`.

Status: inactive placeholder.
