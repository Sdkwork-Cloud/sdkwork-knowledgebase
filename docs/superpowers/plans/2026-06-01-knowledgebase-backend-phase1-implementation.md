# Knowledgebase Backend Phase 1 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build the first backend foundation for `sdkwork-knowledgebase`: a Rust workspace with drive-first storage boundaries, LLM Wiki standard file rendering, schema migration skeletons, OpenAPI skeletons, and verification tests.

**Architecture:** Phase 1 creates the reusable backend component base, not the full product workflow. Business file bytes must flow only through `sdkwork-drive` abstractions. LLM Wiki compatibility is established through contract types and renderers for `AGENTS.md`, `wiki/index.md`, `wiki/log.md`, and local mirror manifest structures.

**Tech Stack:** Rust 2021, Cargo workspace, `serde`, `serde_json`, `thiserror`, `async-trait`, `tokio`, `uuid`, `time`, `sha2`, `utoipa`-ready OpenAPI JSON skeletons, `sdkwork-drive-storage-contract` as the only lower-level storage dependency.

---

## Scope

This plan implements the first working backend foundation only.

Included:

- Rust workspace and crate structure aligned with `sdkwork-claw-router`.
- Reusable contract crate for DTOs, IDs, enums, and operation IDs.
- Product crate with ports and pure domain services.
- Drive adapter crate boundary that depends on `sdkwork-drive-storage-contract`.
- Test support crate with fake in-memory drive storage.
- LLM Wiki schema/index/log renderers.
- Local mirror manifest and delta manifest model types.
- SQL migration skeleton files for PostgreSQL and SQLite.
- OpenAPI skeleton JSON files under `sdks`.
- Verification scripts and tests.

Excluded from Phase 1:

- Frontend UI.
- apps package implementation.
- Full parser/OCR/vector/embedding adapters.
- Real HTTP server routes.
- Full SDK generation output.
- Full ingestion worker runtime.

## File Structure

Create:

- `Cargo.toml` - workspace definition and shared dependencies.
- `rust-toolchain.toml` - Rust channel pin.
- `.gitignore` - target and generated artifact exclusions.
- `crates/sdkwork-knowledgebase-contract` - API/domain contracts.
- `crates/sdkwork-knowledgebase-core` - pure domain utilities and validation.
- `crates/sdkwork-knowledgebase-drive` - only crate allowed to call `sdkwork-drive-storage-contract`.
- `crates/sdkwork-knowledgebase-test-support` - fake drive store and fixtures.
- `services/sdkwork-knowledgebase-product` - product services, ports, and renderers.
- `services/sdkwork-knowledgebase-storage-sqlx` - migration registry and SQLx-facing schema placeholders.
- `sdks/sdkwork-knowledgebase-app-sdk/openapi/knowledgebase-app-api.openapi.json`
- `sdks/sdkwork-knowledgebase-backend-sdk/openapi/knowledgebase-backend-api.openapi.json`
- `tools/verify_phase1.ps1` - local verification command.

Modify:

- `docs/superpowers/specs/2026-06-01-knowledgebase-backend-design.md` only if implementation reveals a design correction.

## Task 1: Workspace Scaffold

**Files:**
- Create: `Cargo.toml`
- Create: `rust-toolchain.toml`
- Create: `.gitignore`
- Create: crate and service directories listed above.

- [ ] **Step 1: Write workspace skeleton**

Create a Cargo workspace with these members:

```toml
[workspace]
members = [
    "crates/sdkwork-knowledgebase-contract",
    "crates/sdkwork-knowledgebase-core",
    "crates/sdkwork-knowledgebase-drive",
    "crates/sdkwork-knowledgebase-test-support",
    "services/sdkwork-knowledgebase-product",
    "services/sdkwork-knowledgebase-storage-sqlx",
]
resolver = "2"

[workspace.package]
edition = "2021"
version = "0.1.0"
license = "AGPL-3.0-or-later"

[workspace.dependencies]
async-trait = "0.1"
bytes = "1"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
sha2 = "0.10"
thiserror = "1"
time = { version = "0.3", features = ["serde", "formatting", "parsing"] }
tokio = { version = "1", features = ["macros", "rt-multi-thread", "sync"] }
uuid = { version = "1", features = ["serde", "v4"] }
```

- [ ] **Step 2: Add minimal `lib.rs` for each crate**

Each crate should compile with a focused module tree and no placeholder panics.

- [ ] **Step 3: Run metadata check**

Run: `cargo metadata --no-deps`

Expected: command exits successfully and lists all six workspace packages.

## Task 2: Contract Crate

**Files:**
- Create: `crates/sdkwork-knowledgebase-contract/Cargo.toml`
- Create: `crates/sdkwork-knowledgebase-contract/src/lib.rs`
- Create: `crates/sdkwork-knowledgebase-contract/src/ids.rs`
- Create: `crates/sdkwork-knowledgebase-contract/src/enums.rs`
- Create: `crates/sdkwork-knowledgebase-contract/src/wiki.rs`
- Create: `crates/sdkwork-knowledgebase-contract/src/mirror.rs`
- Create: `crates/sdkwork-knowledgebase-contract/src/operations.rs`
- Test: `crates/sdkwork-knowledgebase-contract/tests/operation_ids.rs`
- Test: `crates/sdkwork-knowledgebase-contract/tests/llm_wiki_contract.rs`

- [ ] **Step 1: Write failing operation ID tests**

Tests must assert:

```rust
assert_eq!(WIKI_INDEX_RETRIEVE, "wiki.index.retrieve");
assert_eq!(WIKI_LOG_ENTRIES_CREATE, "wiki.log.entries.create");
assert_eq!(WIKI_SCHEMA_PROFILES_CREATE, "wiki.schema.profiles.create");
assert!(!ALL_OPERATION_IDS.iter().any(|id| id.contains('_')));
assert!(!ALL_OPERATION_IDS.iter().any(|id| id.starts_with("wikiIndex")));
```

- [ ] **Step 2: Implement operation constants**

Implement all Phase 1 operation IDs from the spec.

- [ ] **Step 3: Write failing LLM Wiki model tests**

Tests must assert required paths:

```rust
assert_eq!(LlmWikiPaths::default().agents_md, "wiki/schema/AGENTS.md");
assert_eq!(LlmWikiPaths::default().index_md, "wiki/index.md");
assert_eq!(LlmWikiPaths::default().log_md, "wiki/log.md");
```

- [ ] **Step 4: Implement contract types**

Implement:

- `KnowledgeSpaceId`
- `KnowledgePageId`
- `DriveObjectRefId`
- `WikiPageType`
- `WikiCandidateType`
- `WikiLogEventType`
- `LlmWikiPaths`
- `WikiPageSummary`
- `WikiLogEntry`
- `MirrorManifest`
- `DeltaManifest`

- [ ] **Step 5: Run contract tests**

Run: `cargo test -p sdkwork-knowledgebase-contract`

Expected: all tests pass.

## Task 3: Product Ports and Drive Boundary

**Files:**
- Create: `services/sdkwork-knowledgebase-product/Cargo.toml`
- Create: `services/sdkwork-knowledgebase-product/src/lib.rs`
- Create: `services/sdkwork-knowledgebase-product/src/ports/mod.rs`
- Create: `services/sdkwork-knowledgebase-product/src/ports/knowledge_drive_storage.rs`
- Create: `crates/sdkwork-knowledgebase-drive/Cargo.toml`
- Create: `crates/sdkwork-knowledgebase-drive/src/lib.rs`
- Create: `crates/sdkwork-knowledgebase-drive/src/adapter.rs`
- Test: `services/sdkwork-knowledgebase-product/tests/drive_boundary.rs`

- [ ] **Step 1: Write failing boundary tests**

Tests must prove product services depend on `KnowledgeDriveStorage`, not `sdkwork-drive-storage-contract`.

- [ ] **Step 2: Define product storage port**

Define:

```rust
#[async_trait]
pub trait KnowledgeDriveStorage: Send + Sync {
    async fn put_object(&self, request: PutKnowledgeObjectRequest) -> Result<KnowledgeObjectRef, KnowledgeStorageError>;
    async fn get_object_text(&self, object_ref: &KnowledgeObjectRef) -> Result<String, KnowledgeStorageError>;
}
```

- [ ] **Step 3: Implement drive adapter skeleton**

`sdkwork-knowledgebase-drive` is the only Phase 1 crate that may depend on:

```toml
sdkwork-drive-storage-contract = { path = "../../sdkwork-drive/crates/sdkwork-drive-storage-contract" }
```

If that relative path does not resolve from this repo, use the absolute local path in `Cargo.toml` and document it in a comment.

- [ ] **Step 4: Run boundary tests**

Run: `cargo test -p sdkwork-knowledgebase-product drive_boundary`

Expected: tests pass.

## Task 4: Test Support Fake Drive

**Files:**
- Create: `crates/sdkwork-knowledgebase-test-support/Cargo.toml`
- Create: `crates/sdkwork-knowledgebase-test-support/src/lib.rs`
- Create: `crates/sdkwork-knowledgebase-test-support/src/fake_drive.rs`
- Test: `crates/sdkwork-knowledgebase-test-support/tests/fake_drive.rs`

- [ ] **Step 1: Write failing fake drive tests**

Test put/read/checksum behavior:

```rust
let drive = FakeKnowledgeDriveStorage::default();
let object_ref = drive.put_text("wiki/index.md", "# Index").await.unwrap();
assert_eq!(drive.read_text(&object_ref).await.unwrap(), "# Index");
assert_eq!(object_ref.logical_path, "wiki/index.md");
```

- [ ] **Step 2: Implement fake drive**

Use an in-memory `Arc<Mutex<HashMap<String, StoredObject>>>`.

- [ ] **Step 3: Run tests**

Run: `cargo test -p sdkwork-knowledgebase-test-support`

Expected: all tests pass.

## Task 5: LLM Wiki Renderers

**Files:**
- Create: `services/sdkwork-knowledgebase-product/src/wiki/mod.rs`
- Create: `services/sdkwork-knowledgebase-product/src/wiki/schema_renderer.rs`
- Create: `services/sdkwork-knowledgebase-product/src/wiki/index_renderer.rs`
- Create: `services/sdkwork-knowledgebase-product/src/wiki/log_renderer.rs`
- Test: `services/sdkwork-knowledgebase-product/tests/llm_wiki_renderers.rs`

- [ ] **Step 1: Write failing renderer tests**

Tests must assert:

- `AGENTS.md` mentions raw sources, wiki, schema, ingest, query, lint, and `sdkwork-drive`.
- `index.md` contains category headings and wikilinks.
- `log.md` uses `## [timestamp] event | title` headings.
- Rendered files are written through `KnowledgeDriveStorage`.

- [ ] **Step 2: Implement renderers**

Implement pure renderers returning `String`, plus service functions that persist through the storage port.

- [ ] **Step 3: Run renderer tests**

Run: `cargo test -p sdkwork-knowledgebase-product llm_wiki_renderers`

Expected: all tests pass.

## Task 6: Local Mirror Manifest Models

**Files:**
- Modify: `crates/sdkwork-knowledgebase-contract/src/mirror.rs`
- Test: `crates/sdkwork-knowledgebase-contract/tests/local_mirror_manifest.rs`

- [ ] **Step 1: Write failing manifest serialization tests**

Tests must assert JSON contains:

- `llmWikiCompatibility.profile`
- `llmWikiCompatibility.agentInstructionPath`
- `llmWikiCompatibility.indexPath`
- `llmWikiCompatibility.logPath`
- `contentPolicy.includeRawSources`

- [ ] **Step 2: Implement manifest structs**

Use serde with lowerCamelCase fields.

- [ ] **Step 3: Run contract tests**

Run: `cargo test -p sdkwork-knowledgebase-contract local_mirror_manifest`

Expected: all tests pass.

## Task 7: SQL Migration Skeleton

**Files:**
- Create: `services/sdkwork-knowledgebase-storage-sqlx/Cargo.toml`
- Create: `services/sdkwork-knowledgebase-storage-sqlx/src/lib.rs`
- Create: `services/sdkwork-knowledgebase-storage-sqlx/src/migrations.rs`
- Create: `services/sdkwork-knowledgebase-storage-sqlx/migrations/postgres/V202606010001__knowledgebase_core.sql`
- Create: `services/sdkwork-knowledgebase-storage-sqlx/migrations/sqlite/V202606010001__knowledgebase_core.sql`
- Test: `services/sdkwork-knowledgebase-storage-sqlx/tests/migration_manifest.rs`

- [ ] **Step 1: Write failing migration manifest tests**

Tests must assert migration text contains:

- `kb_space`
- `kb_drive_object_ref`
- `kb_wiki_page`
- `kb_wiki_schema_profile`
- `kb_wiki_log_entry`
- `kb_local_mirror_package`

- [ ] **Step 2: Add migration skeletons**

Use SDKWork-compatible table names and common columns. Full schema detail can expand in later phases, but Phase 1 must include all key table names.

- [ ] **Step 3: Run storage tests**

Run: `cargo test -p sdkwork-knowledgebase-storage-sqlx`

Expected: all tests pass.

## Task 8: OpenAPI Skeletons

**Files:**
- Create: `sdks/sdkwork-knowledgebase-app-sdk/openapi/knowledgebase-app-api.openapi.json`
- Create: `sdks/sdkwork-knowledgebase-backend-sdk/openapi/knowledgebase-backend-api.openapi.json`
- Create: `tools/verify_openapi_operation_ids.ps1`

- [ ] **Step 1: Write OpenAPI skeletons**

Include Phase 1 paths and operation IDs from the spec, especially:

- `wiki.index.retrieve`
- `wiki.log.retrieve`
- `wiki.log.entries.create`
- `wiki.schema.retrieve`
- `wiki.schema.profiles.create`
- `wiki.queries.fileAnswer`

- [ ] **Step 2: Write verification script**

Script must fail if:

- any `operationId` contains `_`
- any `operationId` starts with `wikiIndex`, `wikiLog`, `wikiSchema`, or `wikiPages`
- required `wiki.*` operation IDs are missing

- [ ] **Step 3: Run verification**

Run: `powershell -ExecutionPolicy Bypass -File tools/verify_openapi_operation_ids.ps1`

Expected: exits successfully.

## Task 9: Phase Verification Script

**Files:**
- Create: `tools/verify_phase1.ps1`

- [ ] **Step 1: Create verification script**

Script runs:

```powershell
cargo fmt --all --check
cargo test --workspace
powershell -ExecutionPolicy Bypass -File tools/verify_openapi_operation_ids.ps1
```

- [ ] **Step 2: Run verification**

Run: `powershell -ExecutionPolicy Bypass -File tools/verify_phase1.ps1`

Expected: all commands pass.

## Task 10: Documentation Checkpoint

**Files:**
- Modify: `README.md`

- [ ] **Step 1: Create backend README**

Document:

- workspace structure
- drive-first storage rule
- LLM Wiki standard files
- local verification command
- no frontend/apps in Phase 1

- [ ] **Step 2: Run final verification**

Run: `powershell -ExecutionPolicy Bypass -File tools/verify_phase1.ps1`

Expected: all commands pass.

## Implementation Notes

- Use `apply_patch` for manual edits.
- Do not add frontend code.
- Do not store file bytes in SQL.
- Do not create direct filesystem/S3/MinIO storage paths in product logic.
- Only `crates/sdkwork-knowledgebase-drive` may depend on `sdkwork-drive-storage-contract`.
- Keep all generated SDK output out of Phase 1 unless the SDK generator is explicitly run later.

---

## Phase 2 Extension: Space Initialization and Ingest Foundation

**Goal:** Add the first usable backend services on top of the Phase 1 foundation: create a knowledge space, initialize LLM Wiki standard files through drive, and model ingest jobs with an idempotent state machine.

Included:

- `KnowledgeSpaceService` pure service.
- `KnowledgeSpaceStore` and in-memory test implementation.
- `KnowledgeWikiInitializerService` that persists `AGENTS.md`, `wiki/index.md`, and `wiki/log.md`.
- `KnowledgeIngestionService` with idempotent job creation and state transitions.
- Contract DTOs for space initialization and ingest jobs.
- Additional OpenAPI skeleton paths for spaces and ingests.

Excluded:

- Real SQL repositories.
- Parser/OCR/vector/embedding workers.
- HTTP runtime.

## Task 11: Space Contract and In-Memory Store

**Files:**
- Create: `crates/sdkwork-knowledgebase-contract/src/space.rs`
- Modify: `crates/sdkwork-knowledgebase-contract/src/lib.rs`
- Create: `services/sdkwork-knowledgebase-product/src/ports/knowledge_space_store.rs`
- Modify: `services/sdkwork-knowledgebase-product/src/ports/mod.rs`
- Test: `services/sdkwork-knowledgebase-product/tests/space_initialization.rs`

- [ ] Write failing tests for creating a space and initializing standard LLM Wiki files.
- [ ] Implement `KnowledgeSpace`, `CreateKnowledgeSpaceRequest`, and `KnowledgeSpaceStatus`.
- [ ] Implement `KnowledgeSpaceStore` trait.
- [ ] Add in-test memory store.
- [ ] Run product tests.

## Task 12: Wiki Initializer Service

**Files:**
- Create: `services/sdkwork-knowledgebase-product/src/wiki/initializer.rs`
- Modify: `services/sdkwork-knowledgebase-product/src/wiki/mod.rs`
- Test: `services/sdkwork-knowledgebase-product/tests/space_initialization.rs`

- [ ] Write failing test proving initialization writes `wiki/schema/AGENTS.md`, `wiki/index.md`, and `wiki/log.md` through drive.
- [ ] Implement `KnowledgeWikiInitializerService`.
- [ ] Run product tests.

## Task 13: Ingest Contract and State Machine

**Files:**
- Create: `crates/sdkwork-knowledgebase-contract/src/ingest.rs`
- Modify: `crates/sdkwork-knowledgebase-contract/src/lib.rs`
- Create: `services/sdkwork-knowledgebase-product/src/ports/knowledge_ingestion_job_store.rs`
- Create: `services/sdkwork-knowledgebase-product/src/ingest/mod.rs`
- Create: `services/sdkwork-knowledgebase-product/src/ingest/service.rs`
- Test: `services/sdkwork-knowledgebase-product/tests/ingestion_service.rs`

- [ ] Write failing tests for idempotent ingest job creation.
- [ ] Write failing tests for valid transitions: queued -> running -> succeeded and queued -> failed.
- [ ] Implement ingest job contract and service.
- [ ] Run product tests.

## Task 14: OpenAPI and Verification Update

**Files:**
- Modify: `sdks/sdkwork-knowledgebase-app-sdk/openapi/knowledgebase-app-api.openapi.json`
- Modify: `tools/verify_openapi_operation_ids.ps1`
- Test: `tools/verify_phase1.ps1`

- [ ] Add app operation IDs `spaces.create`, `spaces.retrieve`, `ingests.create`, and `ingests.retrieve`.
- [ ] Add required operation ID checks for these operations.
- [ ] Run final verification.

## Phase 3 Extension: Drive Object Key and File Registry Foundation

**Goal:** Make drive-backed object key planning and file registry rules executable, so business code cannot scatter arbitrary object keys or trust user-supplied filenames.

Included:

- Pure object key planner in `sdkwork-knowledgebase-core`.
- Safe file name normalization.
- Rejection of path traversal and absolute paths.
- Wiki file entry contract and product port.
- File registry service for standard LLM Wiki files.

## Task 15: Object Key Planner

**Files:**
- Create: `crates/sdkwork-knowledgebase-core/src/object_key.rs`
- Modify: `crates/sdkwork-knowledgebase-core/src/lib.rs`
- Test: `crates/sdkwork-knowledgebase-core/tests/object_key_planner.rs`

- [ ] Write failing tests for standard LLM Wiki file paths.
- [ ] Write failing tests for unsafe path rejection.
- [ ] Implement object key planner and safe filename normalization.
- [ ] Run core tests.

## Task 16: Wiki File Entry Registry

**Files:**
- Create: `crates/sdkwork-knowledgebase-contract/src/wiki_file.rs`
- Modify: `crates/sdkwork-knowledgebase-contract/src/lib.rs`
- Create: `services/sdkwork-knowledgebase-product/src/ports/knowledge_wiki_file_entry_store.rs`
- Create: `services/sdkwork-knowledgebase-product/src/wiki/file_registry.rs`
- Modify: `services/sdkwork-knowledgebase-product/src/wiki/mod.rs`
- Test: `services/sdkwork-knowledgebase-product/tests/wiki_file_registry.rs`

- [ ] Write failing tests proving standard files are registered with logical path, role, and drive object ref.
- [ ] Implement contract, port, and registry service.
- [ ] Run product tests.
