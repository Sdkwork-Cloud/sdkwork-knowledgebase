# Apps

Purpose: runnable application surface roots, app shells, and client packages promoted to application surfaces.

Owner: SDKWork Knowledgebase maintainers.

Allowed content: app surface roots that carry their own local dictionary when independently built or launched.

Forbidden content: Rust workspace crates, generated SDK output, runtime user config, secrets, and local build artifacts.

## Active Surfaces

| Surface | Path | Architecture |
| --- | --- | --- |
| PC browser/desktop | `sdkwork-knowledgebase-pc/` | `APP_PC_ARCHITECTURE_SPEC.md` |

Related specs: `../sdkwork-specs/SDKWORK_WORKSPACE_SPEC.md`, `../sdkwork-specs/APP_MANIFEST_SPEC.md`, `../sdkwork-specs/APP_CLIENT_ARCHITECTURE_ALIGNMENT_SPEC.md`, `../sdkwork-specs/APP_PC_ARCHITECTURE_SPEC.md`.

Verification: `pnpm verify` from repository root.
