# Knowledgebase Production And Commercial Readiness Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Remove the audited runtime, fake-success, lifecycle, concurrency, pagination, database, deployment, and documentation gaps so the current prelaunch application has an evidence-based production release path.

**Architecture:** Preserve the existing composed SDK, service-port, repository, and worker boundaries. Convert synchronous or fake completion paths into typed failures or durable asynchronous operations; keep all collection work bounded at the store; make lifecycle transitions recoverable and idempotent; align deployment descriptors with metrics actually exported by the runtime.

**Tech Stack:** React 19, Vite 6, TypeScript, Rust, Tokio, Axum, SQLx Any/PostgreSQL/SQLite, Kubernetes, OpenAPI 3.1, generated SDKWork TypeScript SDKs.

---

## Execution Rules

- Preserve unrelated dirty-worktree changes and do not edit generated SDK transport output directly.
- Use test-first red/green cycles for authored behavior. Configuration-only changes use validator and render checks.
- Keep public HTTP inputs and outputs on the SDKWork v3 operation/envelope/pagination profiles.
- Avoid schema migrations where an existing durable indexed lifecycle field can express the requirement. Any unavoidable migration, auth semantic change, or production deployment governance change remains a human-review checkpoint.
- After each task, run the narrow test first. After each phase, run the affected SDKWork validators.

### Task 1: Restore A Single Router Runtime

**Files:**
- Modify: `apps/sdkwork-knowledgebase-pc/vite.config.ts`
- Modify: `apps/sdkwork-knowledgebase-pc/package.json` only if dependency convergence requires it
- Test: `apps/sdkwork-knowledgebase-pc/e2e/shell.smoke.spec.ts`

- [ ] Record the existing Playwright `useLocation` failure.
- [ ] Make `react-router` and `react-router-dom` resolve to the PC application's canonical dependency instance.
- [ ] Run the shell smoke tests and the full four-test Playwright suite.

### Task 2: Remove Fake Agent And Media Success

**Files:**
- Modify: `apps/sdkwork-knowledgebase-pc/packages/sdkwork-knowledgebase-pc-knowledgebase/src/services/knowledgeAgentChatService.ts`
- Modify: `apps/sdkwork-knowledgebase-pc/packages/sdkwork-knowledgebase-pc-knowledgebase/src/services/knowledgeSpaceSettingsService.ts`
- Modify: `crates/sdkwork-routes-knowledgebase-app-api/src/hosted_commerce.rs`
- Test: adjacent TypeScript and Rust service tests

- [ ] Add tests proving production defaults do not select the contract agent or advertise a different provider/model.
- [ ] Add tests proving transcription and image generation fail explicitly when no real result is available.
- [ ] Select the registered production Rig implementation and provider configuration, or return a typed unavailable error when provider configuration is absent.
- [ ] Remove URL-guess transcription and stock-image fallbacks.

### Task 3: Retired Prelaunch Publication Path

Status: superseded and completed by the Live Wiki clean baseline.

The unreleased Knowledgebase-owned publication implementation, legacy public-site configuration,
permissions, API/SDK surface, and UI were removed rather than repaired. Do not restore the deleted
commerce publication path. New public Wiki work follows
`docs/product/requirements/REQ-2026-0721-live-wiki-cloud-publication.md` and the shared
Deploy/Web Server descriptor/provider architecture.

### Task 4: Close Upload And Ingest Lifecycle Leaks

**Files:**
- Modify: `crates/sdkwork-intelligence-knowledgebase-service/src/ingest/upload_session.rs`
- Modify: `crates/sdkwork-intelligence-knowledgebase-service/src/ingest/api_markdown_ingest_pipeline.rs`
- Modify: `crates/sdkwork-intelligence-knowledgebase-service/src/ports/knowledge_ingestion_job_store.rs`
- Modify: `crates/sdkwork-intelligence-knowledgebase-repository-sqlx/src/sqlite_import_stores.rs`
- Modify: `crates/sdkwork-routes-knowledgebase-app-api/src/hosted_upload.rs`
- Test: service and SQLite repository tests

- [ ] Add tests for deterministic expiry reconstruction, expired-session rejection, and stale queued/running recovery.
- [ ] Add tests for Drive put and linkage failures transitioning the job to failed.
- [ ] Implement one failure guard around all post-`mark_running` work.
- [ ] Reap stale upload-session jobs in bounded indexed batches before quota checks.

### Task 5: Make Audit Delivery Durable And Bounded

**Files:**
- Modify: `crates/sdkwork-intelligence-knowledgebase-repository-sqlx/src/audit_event_store.rs`
- Modify: runtime/bootstrap call sites that emit audit records
- Test: repository and runtime tests

- [ ] Replace detached per-event spawn with an async result-bearing or bounded durable path.
- [ ] Ensure shutdown flushes accepted events and database errors are observable.
- [ ] Use the existing outbox/transaction mechanism where the audit event belongs to a business mutation.

### Task 6: Bound External HTTP Bodies And Preserve TLS Identity

**Files:**
- Modify: `crates/sdkwork-intelligence-knowledgebase-service/src/ingest/web_link_fetch.rs`
- Modify: `crates/sdkwork-intelligence-knowledgebase-service/src/imports/github_api.rs`
- Test: HTTP adapter tests

- [ ] Add HTTPS host-identity and oversized streaming-body regression tests.
- [ ] Use reqwest DNS resolution/pinning without replacing the URL hostname.
- [ ] Stream chunks into a bounded buffer and stop at the configured maximum.

### Task 7: Make Quotas Atomic And Cover Batch Imports

**Files:**
- Modify: `crates/sdkwork-routes-knowledgebase-app-api/src/tenant_quota_enforcement.rs`
- Modify: `crates/sdkwork-routes-knowledgebase-app-api/src/hosted.rs`
- Modify: repository/service ports as needed
- Test: concurrent quota and Git import tests

- [ ] Add concurrent tests proving only capacity-bounded writers succeed.
- [ ] Reserve and release ingest/document/storage capacity transactionally.
- [ ] Apply reservations to every Git import batch and roll them back on failure.

### Task 8: Align GDPR And OKF Pagination

**Files:**
- Modify: `crates/sdkwork-intelligence-knowledgebase-repository-sqlx/src/audit_event_store.rs`
- Modify: `crates/sdkwork-routes-knowledgebase-app-api/src/hosted_backend.rs`
- Modify: `crates/sdkwork-intelligence-knowledgebase-service/src/knowledge_engine/okf_native.rs`
- Modify: corresponding contracts/OpenAPI only through authority sources
- Test: repository, service, route, pagination validator

- [ ] Add tests with more than 5,000 audit events and more than 200 OKF concepts.
- [ ] Implement cursor/keyset pages at the store and iterate bounded pages for internal search/read operations.
- [ ] Make GDPR export a cursor page or durable async export, and anonymize in bounded batches.

### Task 9: Bound The Synchronous Agent Bridge

**Files:**
- Modify: `crates/sdkwork-knowledgebase-agent-provider/src/async_bridge.rs`
- Test: bridge concurrency tests

- [ ] Add saturation, timeout, and shutdown tests.
- [ ] Replace the unbounded channel with a bounded queue and typed enqueue/timeout failures.
- [ ] Propagate cancellation instead of panicking on send/receive failure.

### Task 10: Align PostgreSQL Tenant Context And Database Contracts

**Files:**
- Modify: `crates/sdkwork-intelligence-knowledgebase-repository-sqlx/src/db/bootstrap.rs`
- Modify: `crates/sdkwork-intelligence-knowledgebase-repository-sqlx/src/db/postgres_tenant_session.rs`
- Modify: `database/database.manifest.json`
- Modify: `database/contract/schema.yaml`
- Modify: `tools/materialize_knowledgebase_database_contract.mjs`
- Test: PostgreSQL optional integration and database validators

- [ ] Make manifest/materializer/schema contract versions identical and validator-detectable.
- [ ] Add request-scoped tenant checkout/reset integration tests before enabling shared-pool claims.
- [ ] Retain deployment-bound mode until shared mode has real PostgreSQL evidence.

### Task 11: Make Cluster Descriptors Executable

**Files:**
- Modify: `deployments/kubernetes/networkpolicy.yaml`
- Modify: `deployments/kubernetes/hpa.yaml`
- Modify: observability metric exporters and deployment docs
- Test: deployment/static metrics verification scripts

- [ ] Remove nonexistent custom metrics or export them with matching adapter rules.
- [ ] Represent public HTTPS egress with an enforceable CNI/domain proxy policy or document the required platform egress gateway.
- [ ] Validate YAML, selectors, probes, PDB, HPA, rollout, and rollback assumptions.

### Task 12: Remove Identifier Collision Risk

**Files:**
- Modify: deployment identity configuration and `crates/sdkwork-intelligence-knowledgebase-repository-sqlx/src/id.rs`
- Test: identifier configuration tests

- [ ] Reject hashed orchestration names in production.
- [ ] Require a collision-free assigned Snowflake node id or use the approved database/id service authority.

### Task 13: Make Tests Exercise Real Contracts

**Files:**
- Modify: `apps/sdkwork-knowledgebase-pc/e2e/knowledgeApiMocks.ts`
- Modify: readiness/security tests that only assert source strings
- Test: Playwright, live smoke, SQLite and PostgreSQL integration suites

- [ ] Make mocks use SDKWork envelopes and correct 201/202/204/ProblemDetail semantics.
- [ ] Replace important regex assertions with behavior tests.
- [ ] Keep live PostgreSQL/API tests mandatory for release while skippable for local development with explicit skip output.

### Task 14: Synchronize Product, Architecture, And Operations Documentation

**Files:**
- Modify: `docs/product/prd/PRD.md`
- Modify: `docs/product/prd/PRD-mvp-launch.md`
- Modify: `docs/product/prd/PRD-phase2-commercial-saas.md`
- Modify: `docs/architecture/tech/TECH_ARCHITECTURE.md`
- Modify: deployment/runbook docs and `sdkwork.app.config.json`

- [ ] Remove completed checkmarks that lack behavior evidence.
- [ ] Document only implemented runtime modes and truthful capability status.
- [ ] Keep publication inactive until PostgreSQL, live smoke, supply-chain, rollout, and rollback evidence exists.

### Task 15: Final Verification And Debt Rescan

- [ ] Run focused TypeScript, Rust, SQLite, PostgreSQL, Playwright, and deployment tests.
- [ ] Run API operation, response envelope, pagination, app SDK import, database, composition, check, lint, build, and verify gates.
- [ ] Rescan for fake success, unbounded body reads, unbounded channels, full-list pagination, stale documentation, and generated-code edits.
- [ ] Record unresolved external evidence without marking the application commercially released.
