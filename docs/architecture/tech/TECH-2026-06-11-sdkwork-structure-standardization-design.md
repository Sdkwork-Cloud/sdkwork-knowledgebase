> Owner: SDKWork Knowledgebase maintainers
>
> **Superseded (2026-06-24):** Structure migration is complete. `apps/sdkwork-knowledgebase-pc/` is the active PC surface; `crates/` holds Rust workspace members. Current architecture: [TECH_ARCHITECTURE.md](TECH_ARCHITECTURE.md).

## Context

This repository is an SDKWork application root because `sdkwork.app.config.json` is present at the repository root. Current root and component metadata were generated before the latest `../sdkwork-specs` directory structure and Rust naming standards. The repository must be brought to the current standard without retaining legacy package names, wrapper crates, or compatibility aliases.

Human approval was given on 2026-06-11 to perform the final-state migration and not preserve old crate/package identities.

## Standards

- `../sdkwork-specs/SOUL.md`
- `../sdkwork-specs/SDKWORK_WORKSPACE_SPEC.md`
- `../sdkwork-specs/AGENTS_SPEC.md`
- `../sdkwork-specs/APP_MANIFEST_SPEC.md`
- `../sdkwork-specs/COMPONENT_SPEC.md`
- `../sdkwork-specs/CODE_STYLE_SPEC.md`
- `../sdkwork-specs/NAMING_SPEC.md`
- `../sdkwork-specs/RUST_CODE_SPEC.md`
- `../sdkwork-specs/API_SPEC.md`
- `../sdkwork-specs/SDK_SPEC.md`
- `../sdkwork-specs/SDK_WORKSPACE_GENERATION_SPEC.md`
- `../sdkwork-specs/MIGRATION_SPEC.md`
- `../sdkwork-specs/TEST_SPEC.md`

## Problems To Eliminate

- `AGENTS.md` claims no `sdkwork.app.config.json` exists, while the repository root has one.
- The root uses `services/` for authored Rust workspace members, but current `SDKWORK_WORKSPACE_SPEC.md` reserves `crates/` for Rust source and does not define a top-level `services/` capability directory.
- Several Rust packages use forbidden or nonstandard responsibility names:
  - `sdkwork-knowledgebase-core`
  - `sdkwork-knowledgebase-product`
  - `sdkwork-knowledgebase-app-api`
  - `sdkwork-knowledgebase-backend-api`
  - `sdkwork-knowledgebase-storage-sqlx`
- Root standard directory placeholders are missing for inactive or newly standardized capability directories.
- `.sdkwork` READMEs still contain template variables.
- The app manifest points `publish.config.workspaceRoot` and `devApp.sourceRoot` to `apps/sdkwork-knowledgebase`, which is absent.
- Verification does not currently fail on the above structural drift.

## Target Shape

The repository root remains the primary application root. All authored Rust packages live under `crates/`. Standard inactive directories exist with tracked README placeholders that state their inactive status and allowed contents.

Rust package final identities:

| Current package | Final package | Responsibility |
| --- | --- | --- |
| `sdkwork-knowledgebase-core` | `sdkwork-intelligence-knowledgebase-object-key-service` | Focused object key planning service/helper boundary |
| `sdkwork-knowledgebase-product` | `sdkwork-intelligence-knowledgebase-service` | Business services, ports, domain use cases, wiki/retrieval/import orchestration |
| `sdkwork-knowledgebase-storage-sqlx` | `sdkwork-intelligence-knowledgebase-repository-sqlx` | SQLx repository implementations and migration registry |
| `sdkwork-knowledgebase-app-api` | `sdkwork-routes-knowledgebase-app-api` | App API route adapter |
| `sdkwork-knowledgebase-backend-api` | `sdkwork-routes-knowledgebase-backend-api` | Backend API route adapter |

No compatibility package aliases or wrapper crates will be added. Downstream code inside this repository will import the final crate names only.

## Directory Design

Standard directories:

- Active with authored content: `.sdkwork/`, `crates/`, `docs/`, `sdks/`, `specs/`, `tools/`
- Added as tracked inactive placeholders: `apis/`, `apps/`, `jobs/`, `plugins/`, `examples/`, `configs/`, `deployments/`, `scripts/`, `tests/`
- Removed after migration: `services/`

The root README will explain that the repository root is the primary app root and `apps/` is currently a placeholder for future secondary app surfaces.

## Component Spec Design

Every moved or renamed component updates:

- `Cargo.toml package.name`
- `specs/component.spec.json component.name`
- `specs/component.spec.json component.root`
- `specs/component.spec.json component.type` where route or repository semantics are clearer
- `specs/component.spec.json component.capability`
- `specs/component.spec.json component.surface` for route crates
- `specs/component.spec.json contracts.routeManifest` for route crates when a route manifest is added
- component `README.md`
- component `specs/README.md`
- verification commands

## Route Manifest Design

The two route crates will expose source route manifest modules and normalized JSON artifacts:

- `crates/sdkwork-routes-knowledgebase-app-api/src/manifest.rs`
- `crates/sdkwork-routes-knowledgebase-backend-api/src/manifest.rs`
- `sdks/_route-manifests/app-api/sdkwork-routes-knowledgebase-app-api.route-manifest.json`
- `sdks/_route-manifests/backend-api/sdkwork-routes-knowledgebase-backend-api.route-manifest.json`

The normalized artifacts will include `kind: sdkwork.route.manifest`, package name, surface, owner, domain, capability, API authority, SDK family, prefix, source crate root/import, and path list. They are evidence for standards validation and SDK workspace traceability; OpenAPI authority files under `sdks/sdkwork-knowledgebase-*-sdk/openapi/` remain the SDK generation source.

## Verification Design

Add `tools/verify_sdkwork_structure.ps1` and call it from `tools/verify_phase1.ps1`.

The structure verifier checks:

- required root standards paths resolve
- root manifest exists and `AGENTS.md` reflects it
- `.sdkwork` README files do not contain template variables
- root standard directories exist
- top-level `services/` is absent
- Cargo workspace members live under `crates/`
- final package names exist
- forbidden legacy package names and Rust import names are absent from active source/config/test files
- component specs match package names and roots
- route manifest artifacts match route crate names and canonical prefixes
- app manifest root references point to `.`

Existing Rust and SDK verification remains:

- `cargo fmt --all --check`
- `cargo test --workspace`
- `cargo clippy --workspace --tests -- -D warnings`
- `powershell -ExecutionPolicy Bypass -File tools/verify_phase1.ps1`

## Migration And Rollback

This is a package migration with no compatibility window because the human owner approved final-state standardization for the current pre-release repository. Rollback is a normal git revert of the migration commit before release. After release, package consumers must use the final package names.

