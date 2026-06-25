> Migrated from `docs/superpowers/plans/2026-06-11-sdkwork-structure-standardization-implementation.md` on 2026-06-24.
> Owner: SDKWork maintainers

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Bring `sdkwork-knowledgebase` to the current `sdkwork-specs` repository structure and Rust naming standards without preserving legacy package identities.

**Architecture:** Treat the repository root as the primary SDKWork application root. Move all authored Rust workspace members under `crates/`, rename nonstandard packages to responsibility-specific names, add machine-readable route manifest evidence, and add a structure verifier so the standard remains executable.

**Tech Stack:** Rust Cargo workspace, PowerShell verification scripts, SDKWork component specs, OpenAPI/SDK workspace metadata.

---

### Task 1: Add Structure Verification

**Files:**
- Create: `tools/verify_sdkwork_structure.ps1`
- Modify: `tools/verify_phase1.ps1`

- [ ] Write a PowerShell verifier for root dictionary, package names, component specs, app manifest roots, and route manifest evidence.
- [ ] Run the verifier before migration and confirm it fails on the known `services/` and legacy package-name issues.
- [ ] Wire the verifier into `tools/verify_phase1.ps1`.

### Task 2: Move Rust Components

**Files:**
- Move: `services/sdkwork-knowledgebase-app-api` to `crates/sdkwork-router-knowledgebase-app-api`
- Move: `services/sdkwork-knowledgebase-backend-api` to `crates/sdkwork-router-knowledgebase-backend-api`
- Move: `services/sdkwork-knowledgebase-product` to `crates/sdkwork-intelligence-knowledgebase-service`
- Move: `services/sdkwork-knowledgebase-storage-sqlx` to `crates/sdkwork-intelligence-knowledgebase-repository-sqlx`
- Move: `crates/sdkwork-knowledgebase-core` to `crates/sdkwork-intelligence-knowledgebase-object-key-service`
- Modify: `Cargo.toml`

- [ ] Verify move targets are inside the repository root.
- [ ] Move directories with native PowerShell `Move-Item -LiteralPath`.
- [ ] Update Cargo workspace members and `[workspace.dependencies]`.
- [ ] Remove empty `services/`.

### Task 3: Rename Packages And Imports

**Files:**
- Modify: moved `Cargo.toml` files
- Modify: Rust source/tests that import renamed crates
- Modify: dependent crate manifests
- Modify: `Cargo.lock` through `cargo update` or normal Cargo regeneration

- [ ] Rename package `sdkwork-knowledgebase-core` to `sdkwork-intelligence-knowledgebase-object-key-service`.
- [ ] Rename package `sdkwork-knowledgebase-product` to `sdkwork-intelligence-knowledgebase-service`.
- [ ] Rename package `sdkwork-knowledgebase-storage-sqlx` to `sdkwork-intelligence-knowledgebase-repository-sqlx`.
- [ ] Rename package `sdkwork-knowledgebase-app-api` to `sdkwork-router-knowledgebase-app-api`.
- [ ] Rename package `sdkwork-knowledgebase-backend-api` to `sdkwork-router-knowledgebase-backend-api`.
- [ ] Replace Rust import names with final snake_case crate names.
- [ ] Run `cargo metadata --no-deps` to catch manifest and package-name errors.

### Task 4: Repair SDKWork Dictionary And Component Specs

**Files:**
- Modify: `AGENTS.md`
- Modify: `README.md`
- Modify: `sdkwork.app.config.json`
- Modify: `.sdkwork/README.md`
- Modify: `.sdkwork/skills/README.md`
- Modify: `.sdkwork/plugins/README.md`
- Modify: moved component `README.md` and `specs/*`
- Create: `apis/README.md`, `apps/README.md`, `jobs/README.md`, `plugins/README.md`, `examples/README.md`, `configs/README.md`, `deployments/README.md`, `scripts/README.md`, `tests/README.md`

- [ ] Update application identity text in `AGENTS.md`.
- [ ] Replace `.sdkwork` template variables with repository-specific text and relative spec paths.
- [ ] Update `sdkwork.app.config.json` root references to `.`.
- [ ] Document active/inactive root layout in `README.md`.
- [ ] Add standard placeholder READMEs for inactive root directories.
- [ ] Update component specs and component READMEs to final package names and paths.

### Task 5: Add Route Manifest Evidence

**Files:**
- Create: `crates/sdkwork-router-knowledgebase-app-api/src/manifest.rs`
- Modify: `crates/sdkwork-router-knowledgebase-app-api/src/lib.rs`
- Create: `crates/sdkwork-router-knowledgebase-backend-api/src/manifest.rs`
- Modify: `crates/sdkwork-router-knowledgebase-backend-api/src/lib.rs`
- Create: `sdks/_route-manifests/app-api/sdkwork-router-knowledgebase-app-api.route-manifest.json`
- Create: `sdks/_route-manifests/backend-api/sdkwork-router-knowledgebase-backend-api.route-manifest.json`

- [ ] Add framework-neutral manifest modules with package/surface/prefix constants.
- [ ] Add normalized JSON artifacts under `sdks/_route-manifests`.
- [ ] Ensure component specs declare route manifests.

### Task 6: Update Verification Script And Docs References

**Files:**
- Modify: `tools/verify_phase1.ps1`
- Modify: `README.md`
- Modify: active docs as needed

- [ ] Replace legacy package lists and paths in `tools/verify_phase1.ps1`.
- [ ] Keep historical design documents only as history; update current root README and verification docs to final names.
- [ ] Confirm no active source/config/test file references legacy names.

### Task 7: Verify And Fix

**Commands:**
- `powershell -ExecutionPolicy Bypass -File tools/verify_sdkwork_structure.ps1`
- `cargo metadata --no-deps`
- `cargo fmt --all --check`
- `cargo test --workspace`
- `cargo clippy --workspace --tests -- -D warnings`
- `powershell -ExecutionPolicy Bypass -File tools/verify_phase1.ps1`

- [ ] Run the commands.
- [ ] Fix failures.
- [ ] Re-run until the commands pass or report an external blocker with exact output.

