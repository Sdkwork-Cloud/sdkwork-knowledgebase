# SDKWork Knowledgebase Standards Alignment Baseline

Status: active  
Owner: SDKWork maintainers  
Updated: 2026-06-29

## Purpose

Pre-launch alignment baseline against `../sdkwork-specs/`. Native composition authority (ADR-20260629) replaces the retired parallel `dependency.composition.json` model.

## Framework Integration

| Framework | Status | Evidence |
|-----------|--------|----------|
| sdkwork-specs | Aligned | `AGENTS.md`, `pnpm check`, `verify-repo.mjs` |
| sdkwork-web-framework | Aligned | Route crates + `web_bootstrap.rs` + envelope mapping |
| sdkwork-database | Aligned | `database/` lifecycle + `pnpm db:*` |
| sdkwork-utils | Aligned | `sdkwork-utils-rust` / `@sdkwork/utils` + strict `check:utils-integration` (no `.trim()` blank bypass) |
| sdkwork-drive | Aligned | `sdkwork-knowledgebase-drive` + `@sdkwork/drive-app-sdk` PC uploads |
| sdkwork-discovery | Deferred | HTTP-only; enable when RPC services ship |

## Native Composition (APP_COMPOSITION_SPEC)

| Authority | Location |
|-----------|----------|
| Workspace dependency graph | Repository root `pnpm-workspace.yaml` (no nested app workspace) |
| Frontend SDK inventory | `apps/sdkwork-knowledgebase-pc/packages/sdkwork-knowledgebase-pc-core/specs/component.spec.json#contracts.sdkDependencies` |
| Runtime SDK base URLs | `sdkwork-knowledgebase-pc-core/src/composition/dependency-runtime.ts` |
| PC app config workspace pointer | `apps/sdkwork-knowledgebase-pc/sdkwork.app.config.json#packages.workspace` → `../../pnpm-workspace.yaml` |
| Core public exports | `sdkwork-knowledgebase-pc-core/package.json#exports` (`.`, `./sdk`, `./modules`, `./host`, `./session`, `./composition`) |
| SDK contract types (capability packages) | `sdkwork-knowledgebase-pc-core/src/sdk/sdkContractTypes.ts` |

Gate: `pnpm check:app-composition` → `node ../sdkwork-specs/tools/verify-repo.mjs --root .`

## Verification Commands

```powershell
pnpm check
pnpm check:app-composition
node ../sdkwork-specs/tools/check-api-response-envelope.mjs --workspace .
pnpm api:materialize:check
pnpm sdk:generate:check
pnpm db:validate
pnpm verify
pnpm test
```

## Pre-Launch Policy

- Root `README.md` declares `repository-kind: application` per `SDKWORK_WORKSPACE_SPEC.md`.
- Blank/trim helpers must import `@sdkwork/utils` directly (no `pc-commons/stringUtils` re-export shim).
- No `specs/dependency.composition.json` or `contracts.dependencyComposition` pointers.
- No raw HTTP in product UI services; use generated app SDK or composed facades.
- No persistent file bytes outside `sdkwork-drive`.
- No local filesystem bundle discovery in production Rust libraries (test fixtures under `tests/support/` only).
- Demo/offline UX is development-only via `shouldUseKnowledgebaseDemoFallback()`; production builds fail closed.

## Verification Status (2026-06-29)

All gates green on repository root:

| Gate | Result |
|------|--------|
| `pnpm check` | pass |
| `pnpm verify` | pass |
| `verify-repo.mjs` | pass |
| API envelope checker | pass |
| Phase 1 launch readiness | pass |
| Phase 2 commercial readiness | pass |

Dead bootstrap re-export shims under `apps/sdkwork-knowledgebase-pc/src/bootstrap/` were removed; account/session helpers live in `sdkwork-knowledgebase-pc-core/session` and `account/accountViewModel.ts`.
