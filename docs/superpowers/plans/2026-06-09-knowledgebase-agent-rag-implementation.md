# Knowledgebase Agent RAG Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add standard SDKWork Knowledgebase RAG, knowledge-agent profile, API, database, and agent-kernel adapter foundations.

**Architecture:** Keep kernel provider-neutral, make `sdkwork-knowledgebase` the knowledge product authority, and expose a thin adapter crate for `sdkwork_agent_kernel::KnowledgeProvider`. HTTP app/backend APIs are owner-only OpenAPI contracts and generated SDK input.

**Tech Stack:** Rust 2021, Axum, SQLx migration strings, OpenAPI 3.1 JSON, SDKWork generated TypeScript SDKs.

---

### Task 1: Contract And Operation IDs

**Files:**
- Create: `crates/sdkwork-knowledgebase-contract/src/rag.rs`
- Modify: `crates/sdkwork-knowledgebase-contract/src/lib.rs`
- Modify: `crates/sdkwork-knowledgebase-contract/src/operations.rs`
- Test: `crates/sdkwork-knowledgebase-contract/tests/rag_contract.rs`
- Test: `crates/sdkwork-knowledgebase-contract/tests/operation_ids.rs`

- [ ] **Step 1: Write failing contract tests**
  Add tests that construct `KnowledgeRetrievalRequest`, `KnowledgeRetrievalResult`, `KnowledgeContextPack`, `KnowledgeAgentProfile`, and `KnowledgeAgentBinding`.

- [ ] **Step 2: Run red test**
  Run: `cargo test -p sdkwork-knowledgebase-contract rag_contract operation_ids`
  Expected: FAIL because RAG DTOs and operation ids do not exist.

- [ ] **Step 3: Implement DTOs and operation constants**
  Add focused RAG/agent structs and constants without product logic.

- [ ] **Step 4: Run green test**
  Run: `cargo test -p sdkwork-knowledgebase-contract`
  Expected: PASS.

### Task 2: Database Schema

**Files:**
- Modify: `services/sdkwork-knowledgebase-storage-sqlx/migrations/sqlite/V202606010001__knowledgebase_core.sql`
- Modify: `services/sdkwork-knowledgebase-storage-sqlx/migrations/postgres/V202606010001__knowledgebase_core.sql`
- Test: `services/sdkwork-knowledgebase-storage-sqlx/tests/migration_manifest.rs`

- [ ] **Step 1: Write failing migration tests**
  Require `kb_chunk`, `kb_index`, `kb_embedding`, `kb_retrieval_profile`, `kb_retrieval_trace`, `kb_retrieval_hit`, `kb_agent_profile`, and `kb_agent_knowledge_binding`.

- [ ] **Step 2: Run red test**
  Run: `cargo test -p sdkwork-knowledgebase-storage-sqlx migration_manifest`
  Expected: FAIL because tables and indexes are missing.

- [ ] **Step 3: Add portable SQL**
  Add SQLite and PostgreSQL DDL with `kb_`, `uk_kb_`, and `idx_kb_` names, tenant isolation, Snowflake ids, status, timestamps, and versions.

- [ ] **Step 4: Run green test**
  Run: `cargo test -p sdkwork-knowledgebase-storage-sqlx migration_manifest`
  Expected: PASS.

### Task 3: App API RAG And Agent Routes

**Files:**
- Modify: `services/sdkwork-knowledgebase-app-api/src/lib.rs`
- Modify: `services/sdkwork-knowledgebase-app-api/tests/app_openapi_routes.rs`
- Modify: `sdks/sdkwork-knowledgebase-app-sdk/openapi/knowledgebase-app-api.openapi.json`

- [ ] **Step 1: Write failing API tests**
  Assert app OpenAPI exposes retrieval, context pack, agent profile, binding, and preview operations.

- [ ] **Step 2: Run red test**
  Run: `cargo test -p sdkwork-knowledgebase-app-api`
  Expected: FAIL because app OpenAPI/routes are missing.

- [ ] **Step 3: Add trait methods and routes**
  Add app route boundary methods returning default RFC 9457 `501` until product services are wired.

- [ ] **Step 4: Update owner OpenAPI**
  Add owner-only app OpenAPI paths and schemas. Do not edit generated SDK output.

- [ ] **Step 5: Run green test**
  Run: `cargo test -p sdkwork-knowledgebase-app-api`
  Expected: PASS.

### Task 4: Backend API Admin Routes

**Files:**
- Modify: `services/sdkwork-knowledgebase-backend-api/src/lib.rs`
- Modify: `services/sdkwork-knowledgebase-backend-api/tests/backend_openapi_routes.rs`
- Modify: `sdks/sdkwork-knowledgebase-backend-sdk/openapi/knowledgebase-backend-api.openapi.json`

- [ ] **Step 1: Write failing API tests**
  Assert backend OpenAPI exposes index, retrieval profile, trace, and provider health operations.

- [ ] **Step 2: Run red test**
  Run: `cargo test -p sdkwork-knowledgebase-backend-api`
  Expected: FAIL because backend OpenAPI/routes are missing.

- [ ] **Step 3: Add trait methods and routes**
  Add backend route boundary methods returning default `501` until concrete services are wired.

- [ ] **Step 4: Update owner OpenAPI**
  Add owner-only backend OpenAPI paths and schemas. Do not edit generated SDK output.

- [ ] **Step 5: Run green test**
  Run: `cargo test -p sdkwork-knowledgebase-backend-api`
  Expected: PASS.

### Task 5: Agent Provider Adapter Foundation

**Files:**
- Create: `crates/sdkwork-knowledgebase-agent-provider/Cargo.toml`
- Create: `crates/sdkwork-knowledgebase-agent-provider/src/lib.rs`
- Modify: `Cargo.toml`
- Test: `crates/sdkwork-knowledgebase-agent-provider/tests/provider_contract.rs`

- [ ] **Step 1: Write failing adapter tests**
  Test provider manifest, request mapping, retrieval result mapping, document read, and list behavior.

- [ ] **Step 2: Run red test**
  Run: `cargo test -p sdkwork-knowledgebase-agent-provider`
  Expected: FAIL because the crate does not exist.

- [ ] **Step 3: Implement adapter**
  Define a `KnowledgebaseRetrievalClient` trait and `SdkworkKnowledgebaseProvider` implementing `sdkwork_agent_kernel::KnowledgeProvider`.

- [ ] **Step 4: Run green test**
  Run: `cargo test -p sdkwork-knowledgebase-agent-provider`
  Expected: PASS.

### Task 6: Verification

**Files:**
- Modify docs/README only if the public behavior description changes.

- [ ] **Step 1: Format**
  Run: `cargo fmt --all --check`
  Expected: PASS.

- [ ] **Step 2: Workspace tests**
  Run: `cargo test --workspace`
  Expected: PASS.

- [ ] **Step 3: SDKWork checks**
  Run: `powershell -ExecutionPolicy Bypass -File tools/verify_phase1.ps1`
  Expected: PASS or a precise failure list if generated SDK regeneration is required.
