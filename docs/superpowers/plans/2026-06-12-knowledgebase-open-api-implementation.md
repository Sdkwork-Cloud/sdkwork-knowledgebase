# Knowledgebase Open API Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add the Knowledgebase public open-api surface at `/knowledge/v3/api`, generate the owner-only `sdkwork-knowledgebase-sdk` family metadata, and register the surface in `sdkwork-api-cloud-gateway`.

**Architecture:** Implement a dedicated Rust route crate for the public API, materialize a normalized open-api route manifest and owner-only OpenAPI authority, then extend the SDK standardization script and gateway dependency-surface registry. The route crate delegates to existing service traits and keeps app/backend-only behavior out of the public surface.

**Tech Stack:** Rust Cargo workspace, Axum route crates, SDKWork route manifests, OpenAPI 3.x JSON, Node SDK metadata tooling, PowerShell verification, `sdkwork-api-cloud-gateway` Rust config/runtime tests.

---

### Task 1: Characterize Existing Reusable Service Boundaries

**Files:**
- Read: `crates/sdkwork-router-knowledgebase-app-api/src/*.rs`
- Read: `crates/sdkwork-router-knowledgebase-backend-api/src/*.rs`
- Read: `crates/sdkwork-intelligence-knowledgebase-service/src/**/*.rs`
- Read: `crates/sdkwork-knowledgebase-contract/src/**/*.rs`

- [ ] Inspect app route traits and adapters for retrieval, ingestion, context pack, document read, and browser operations.
- [ ] Inspect service traits to confirm no open route needs SQLx or generated SDK dependencies.
- [ ] Identify DTOs already shared through `sdkwork-knowledgebase-contract`.
- [ ] Record the exact operations that can be delegated without adding persistence changes.

### Task 2: Add Failing Open Route Crate Tests

**Files:**
- Create: `crates/sdkwork-router-knowledgebase-open-api/tests/open_api_routes.rs`
- Create: `crates/sdkwork-router-knowledgebase-open-api/Cargo.toml` only after the test target is defined

- [ ] Write a failing test that imports `sdkwork_router_knowledgebase_open_api::manifest` and asserts package, surface, authority, SDK family, prefix, and the approved route list.
- [ ] Write failing HTTP route tests for representative operations:
  - `POST /knowledge/v3/api/retrievals`
  - `GET /knowledge/v3/api/retrievals/{retrievalId}`
  - `POST /knowledge/v3/api/context_packs`
  - `GET /knowledge/v3/api/spaces/{spaceId}/browser`
- [ ] Run `cargo test -p sdkwork-router-knowledgebase-open-api`.
- [ ] Confirm the failure is due to the missing package or missing public API symbols.

### Task 3: Implement Minimal Open Route Crate

**Files:**
- Create: `crates/sdkwork-router-knowledgebase-open-api/Cargo.toml`
- Create: `crates/sdkwork-router-knowledgebase-open-api/README.md`
- Create: `crates/sdkwork-router-knowledgebase-open-api/specs/README.md`
- Create: `crates/sdkwork-router-knowledgebase-open-api/specs/component.spec.json`
- Create: `crates/sdkwork-router-knowledgebase-open-api/src/lib.rs`
- Create: `crates/sdkwork-router-knowledgebase-open-api/src/manifest.rs`
- Create: `crates/sdkwork-router-knowledgebase-open-api/src/paths.rs`
- Create: `crates/sdkwork-router-knowledgebase-open-api/src/routes.rs`
- Create: `crates/sdkwork-router-knowledgebase-open-api/src/handlers.rs`
- Create: `crates/sdkwork-router-knowledgebase-open-api/src/error.rs`
- Create: `crates/sdkwork-router-knowledgebase-open-api/src/ports.rs`
- Create: `crates/sdkwork-router-knowledgebase-open-api/src/adapters.rs`
- Modify: `Cargo.toml`

- [ ] Add the new Cargo workspace member.
- [ ] Add route crate metadata with package name `sdkwork-router-knowledgebase-open-api`.
- [ ] Keep `src/lib.rs` to modules and public re-exports only.
- [ ] Add path constants in `paths.rs` for `/knowledge/v3/api`.
- [ ] Add `manifest.rs` with `surface = "open-api"`, `apiAuthority = "sdkwork-knowledgebase.open"`, `sdkFamily = "sdkwork-knowledgebase-sdk"`, and `auth.mode = "api-key"` semantics.
- [ ] Add service traits in `ports.rs` for the approved public operations.
- [ ] Add `routes.rs` and `handlers.rs` as thin Axum adapters.
- [ ] Add adapters only where needed to reuse existing service trait shapes without leaking app-session context.
- [ ] Run `cargo test -p sdkwork-router-knowledgebase-open-api`.
- [ ] Refactor only after tests pass.

### Task 4: Add Route Manifest Artifact And SDK Family Metadata Tests

**Files:**
- Create: `sdks/_route-manifests/open-api/sdkwork-router-knowledgebase-open-api.route-manifest.json`
- Modify: `sdks/test/verify-sdk-ownership-boundaries.test.mjs`
- Modify: `tools/verify_sdkwork_structure.ps1`
- Modify: `tools/verify_phase1.ps1` if new checks are not already covered

- [ ] Write a failing SDK ownership test that expects `sdkwork-knowledgebase-sdk`.
- [ ] Write a failing structure check that expects the open-api route manifest.
- [ ] Run the narrow test command and confirm it fails on missing open SDK/manifest evidence.
- [ ] Add the normalized manifest artifact with route metadata matching the route crate.
- [ ] Update checks so route manifest, authority, SDK family, prefix, and auth mode are validated.
- [ ] Re-run the narrow checks.

### Task 5: Add OpenAPI Authority And Standardization Support

**Files:**
- Create: `sdks/sdkwork-knowledgebase-sdk/README.md`
- Create: `sdks/sdkwork-knowledgebase-sdk/specs/README.md`
- Create: `sdks/sdkwork-knowledgebase-sdk/specs/component.spec.json`
- Create: `sdks/sdkwork-knowledgebase-sdk/openapi/knowledgebase-open-api.openapi.json`
- Modify: `sdks/standardize-knowledgebase-sdk-family.mjs`

- [ ] Add a failing `node sdks/standardize-knowledgebase-sdk-family.mjs --check` expectation for the missing open SDK family.
- [ ] Add the open family entry to `standardize-knowledgebase-sdk-family.mjs`.
- [ ] Define `apiPrefix: "/knowledge/v3/api"`, `authority: "sdkwork-knowledgebase.open"`, `sdkTarget: "open"`, `packageName: "@sdkwork/knowledgebase-sdk"`, and `primaryClient: "SdkworkKnowledgebaseClient"`.
- [ ] Keep `dependencies: []` unless evidence shows a required dependency SDK for this public surface.
- [ ] Add OpenAPI paths, schemas, security scheme `ApiKey`, operation owner metadata, and problem-detail responses.
- [ ] Run `node sdks/standardize-knowledgebase-sdk-family.mjs`.
- [ ] Run `node sdks/standardize-knowledgebase-sdk-family.mjs --check`.
- [ ] Run `node sdks/test/verify-sdk-ownership-boundaries.test.mjs`.

### Task 6: Generate Open TypeScript SDK If Generator Is Available

**Files:**
- Create or regenerate: `sdks/sdkwork-knowledgebase-sdk/sdkwork-knowledgebase-sdk-typescript/generated/server-openapi/**`

- [ ] Check for `..\sdkwork-sdk-generator\bin\sdkgen.js`.
- [ ] If present, run the canonical generator with `--standard-profile sdkwork-v3` and output under `generated/server-openapi`.
- [ ] If absent, do not create stub generated transport. Record the missing generator as a verification gap and keep OpenAPI/assembly metadata valid.
- [ ] Run generated TypeScript compile if generated output exists and the package has a local compile command.
- [ ] Confirm no generated file is manually edited.

### Task 7: Add Gateway Failing Tests

**Repository:** `E:/sdkwork-space/sdkwork-api-cloud-gateway`

**Files:**
- Modify: `crates/sdkwork-api-cloud-gateway-config/tests/config_tests.rs`
- Modify: `crates/sdkwork-api-cloud-gateway-runtime/tests/runtime_tests.rs`

- [ ] Add a failing config test that expects `SDKWORK_KNOWLEDGEBASE_OPEN_API_BASE_URL`, service id `sdkwork-knowledgebase-open-api`, api authority `sdkwork-knowledgebase.open`, SDK family `sdkwork-knowledgebase-sdk`, and prefix `/knowledge/v3/api`.
- [ ] Add a failing runtime routing test that proves `/knowledge/v3/api/retrievals` resolves to `sdkwork-knowledgebase-open-api`.
- [ ] Run `cargo test -p sdkwork-api-cloud-gateway-config`.
- [ ] Run `cargo test -p sdkwork-api-cloud-gateway-runtime`.
- [ ] Confirm failures are caused by missing Knowledgebase open-api gateway wiring.

### Task 8: Implement Gateway Wiring

**Repository:** `E:/sdkwork-space/sdkwork-api-cloud-gateway`

**Files:**
- Modify: `crates/sdkwork-api-cloud-gateway-config/src/lib.rs`
- Modify: `sdkwork.app.config.json`
- Modify: `specs/component.spec.json`
- Modify: `README.md`
- Modify: `config/sdkwork-api-cloud-gateway.development.toml.example`
- Modify: `config/sdkwork-api-cloud-gateway.test.toml.example`
- Modify: `config/sdkwork-api-cloud-gateway.production.toml.example`

- [ ] Add `KNOWLEDGEBASE_OPEN_API_SERVICE_ID`.
- [ ] Add app manifest environment variable metadata.
- [ ] Add dependency surface and dependency metadata for `/knowledge/v3/api`.
- [ ] Add TOML example upstream entries using `SDKWORK_KNOWLEDGEBASE_OPEN_API_BASE_URL`.
- [ ] Update README service table.
- [ ] Run the gateway config/runtime tests added in Task 7.

### Task 9: Full Knowledgebase Verification

**Repository:** `E:/sdkwork-space/sdkwork-knowledgebase`

**Commands:**
- `cargo test -p sdkwork-router-knowledgebase-open-api`
- `node sdks/standardize-knowledgebase-sdk-family.mjs --check`
- `node sdks/test/verify-sdk-ownership-boundaries.test.mjs`
- `powershell -ExecutionPolicy Bypass -File tools/verify_sdkwork_structure.ps1`
- `powershell -ExecutionPolicy Bypass -File tools/verify_phase1.ps1`
- `cargo fmt --all --check`
- `cargo test --workspace`
- `cargo clippy --workspace --tests -- -D warnings`

- [ ] Run each command and read exit code/output.
- [ ] Fix failures caused by this task.
- [ ] Do not revert unrelated existing workspace changes.
- [ ] Record exact remaining gaps if a toolchain dependency such as `sdkgen` is missing.

### Task 10: Full Gateway Verification

**Repository:** `E:/sdkwork-space/sdkwork-api-cloud-gateway`

**Commands:**
- `cargo test -p sdkwork-api-cloud-gateway-config`
- `cargo test -p sdkwork-api-cloud-gateway-runtime`
- `cargo fmt --all -- --check`
- `cargo test --workspace`

- [ ] Run each command and read exit code/output.
- [ ] Fix failures caused by this task.
- [ ] Preserve unrelated existing gateway changes.
- [ ] Report exact evidence in the final response.
