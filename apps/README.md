# Apps

Purpose: secondary runnable application surface roots, app shells, or demos promoted to application surfaces.

Owner: SDKWork Knowledgebase maintainers.

Allowed content: secondary app surface roots that carry their own manifest and local dictionary when independently built or launched.

Forbidden content: Rust workspace crates, generated SDK output, runtime user config, secrets, and local build artifacts.

Related specs: `../sdkwork-specs/SDKWORK_WORKSPACE_SPEC.md`, `../sdkwork-specs/APP_MANIFEST_SPEC.md`, `../sdkwork-specs/APP_CLIENT_ARCHITECTURE_ALIGNMENT_SPEC.md`.

Verification: `powershell -ExecutionPolicy Bypass -File tools/verify_sdkwork_structure.ps1`.

Status: inactive placeholder. The repository root is the primary SDKWork Knowledgebase application root.
