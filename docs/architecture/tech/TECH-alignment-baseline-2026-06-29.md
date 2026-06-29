# SDKWork Knowledgebase Standards Alignment Baseline

Status: active  
Owner: SDKWork maintainers  
Updated: 2026-06-29

## Purpose

This document records the repository alignment baseline against `../sdkwork-specs/` after pre-launch standardization. Use it as the verification checklist before production release.

## Framework Integration

| Framework | Status | Evidence |
|-----------|--------|----------|
| sdkwork-specs | Aligned | Root `AGENTS.md`, canonical specs in `specs/component.spec.json`, automated `pnpm check` |
| sdkwork-web-framework | Aligned | Route crates use `sdkwork-web-axum`, `web_bootstrap.rs`, `SdkWorkApiResponse` mapping |
| sdkwork-database | Aligned | `database/` lifecycle, `pnpm db:*`, `sdkwork-database-*` in repository-sqlx |
| sdkwork-utils | Aligned | `sdkwork-utils-rust` / `@sdkwork/utils`, `check:utils-integration` |
| sdkwork-drive | Aligned | `sdkwork-knowledgebase-drive` adapter, `@sdkwork/drive-app-sdk` PC uploads |
| sdkwork-discovery | Deferred | HTTP-only phase; `apis/rpc/README.md` placeholder until gRPC services exist |

## Client Dependency Composition

- Manifest: `apps/sdkwork-knowledgebase-pc/specs/dependency.composition.json`
- Core package: `sdkwork-knowledgebase-pc-core/src/composition/`
- Runtime derivation: `buildDependencySdkBaseUrls()` in `dependency-runtime.ts`
- Gate: `pnpm check:dependency-composition`

## Verification Commands

```powershell
pnpm check
pnpm check:dependency-composition
node ../sdkwork-specs/tools/check-api-response-envelope.mjs --workspace .
pnpm api:materialize:check
pnpm sdk:generate:check
pnpm db:validate
pnpm verify
pnpm test
```

## Pre-Launch Policy

- No raw HTTP in product UI services (use generated app SDK or composed facades).
- No persistent file bytes outside `sdkwork-drive` (PC via `@sdkwork/drive-app-sdk`, Rust via `sdkwork-knowledgebase-drive`).
- No local filesystem bundle discovery in production Rust libraries (test fixtures live under `tests/support/` only).
- Demo/offline UX is development-only via `shouldUseKnowledgebaseDemoFallback()`; production builds fail closed.
