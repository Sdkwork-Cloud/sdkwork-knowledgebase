# SDKWork Knowledgebase — MVP Launch Acceptance

Status: active  
Owner: SDKWork maintainers  
Application: sdkwork-knowledgebase  
Updated: 2026-07-05  
Parent: [PRD.md](PRD.md)

## Purpose

Phase 0.1 exit criteria and Phase 1.0 launch acceptance checklist for SDKWork Knowledgebase.

## Phase 0.1 Exit Criteria

### Security

- [x] Public ingress does not expose `/metrics`; Prometheus scrapes via ServiceMonitor only
- [x] Backend OpenAPI declares `x-sdkwork-permission: knowledge.admin` on all protected operations
- [x] Upload session space ACL enforced (`require_space_access` on create/complete)
- [x] Space ACL fail-closed when drive binding missing
- [x] WeChat editor HTML sanitized before insertion
- [x] Production demo/synthetic UI data gated by `shouldUseKnowledgebaseDemoFallback()` (WeChat insert, applet modal, settings avatars, music player, widget templates without external URLs)
- [x] Tenant-scoped dynamic rate limit policy wired from web store (`SqlxDynamicPolicyBundle`) on all HTTP surfaces
- [x] External/catalog adapter engines return `Unsupported` for `list_documents` instead of silent empty lists
- [x] User-facing mutation errors surfaced via `toastKnowledgebaseError` in core KB flows
- [x] `pnpm test:security` passes (tenant isolation, RBAC, audit, demo gating)
- [x] Development secrets are not committed in topology profiles (use `.env.postgres`)

### API & SDK

- [x] `pnpm api:materialize:check` passes (auth-mode, permissions, authority sync)
- [x] `pnpm sdk:generate:check` passes
- [x] Open API included in `verify_openapi_operation_ids.ps1` and phase1 generated SDK roots
- [x] `specs/component.spec.json` indexes all three HTTP surfaces and SDK clients

### Reliability

- [x] Worker `/readyz` probes database connectivity (requires connection pool, not business queries)
- [x] App API `/readyz` simplified to dependency connectivity checks only (see Phase 0.4)
- [x] `list_browser` enforces `ensure_runtime_tenant` like other hosted app routes
- [x] Repository hot paths bounded: chunk load cap, drive ref prefix limit, OKF link list limits
- [x] Agent provider `block_on_async` reuses a dedicated bridge thread/runtime (no per-call OS thread spawn)
- [x] Worker HPA, Service, and ServiceMonitor configured
- [x] K8s manifests include PDB, NetworkPolicy, and `securityContext`
- [x] Ingest pipelines log failures when `mark_failed` cannot persist state
- [x] Production topology documents mandatory Outbox webhook configuration

### Frontend

- [x] No `@packages/` deep imports; package boundary imports only
- [x] Demo/mock fallbacks disabled in production builds (`import.meta.env.PROD`)
- [x] Offline import modals (chat file/dialog, notes) gated by `assertKnowledgebasePreviewFeature` — no synthetic batch writes when API is live
- [x] WeChat save-as-draft persists via `WechatService.publishArticles` (WeChat draft box API)
- [x] AI assistant uses backend agent when `isKnowledgebaseApiAvailable()`; local MCP agent is demo-only
- [x] Image viewer AI tools hidden outside `shouldUseKnowledgebaseDemoFallback()`
- [x] Asset library scan capped (`MAX_ASSET_LIBRARY_ITEMS` / `MAX_ASSET_SCAN_NODES`) with truncation banner
- [x] Asset library modal uses cursor pagination (`listAssetLibraryItemsPage`) with Load more; no synthetic third-party demo assets (API-backed or empty state only)
- [x] Knowledge space members settings use paginated first page; full baseline fetched on save only when members changed
- [x] Partial member sync preserves unloaded baseline members (`buildPartialMemberSyncPayload`); baseline fetched on save only when members changed
- [x] Auto-save and editor uploads surface i18n errors via `toastKnowledgebaseError`; numeric ProblemDetail `60002` maps to tenant quota message
- [x] Editor demo upload uses blob URLs only under `shouldUseKnowledgebaseDemoFallback()`
- [x] Permissions modal uses paginated member count (`20+` when truncated)
- [x] Cloud drive service enforces API availability + network online guards
- [x] WeChat publish/upload/AI stream API failures use `toastKnowledgebaseError` (quota/offline/network aware)
- [x] Network offline fail-closed: `AppShell` wires `setKnowledgebaseNetworkOnline`; mutations call `requireKnowledgebaseNetworkOnline`
- [x] i18n keys for `network.offline` and `feature.previewOnly` error surfaces
- [x] Unimplemented split-view menu removed (no false success toast)
- [x] Document export path re-sanitizes HTML before `innerHTML` assignment
- [x] `pnpm test:frontend` passes; `pnpm lint` (TypeScript) passes
- [x] Ad-hoc root migration scripts removed; `pnpm check:pc-app-hygiene` passes
- [x] Browser/desktop staging and production config examples present

### Verification

```bash
pnpm verify
pnpm test
pnpm lint
```

## Phase 1.0 Launch Acceptance

### Functional

- [x] Author scenario: login → create note → edit document → auto-save (`e2e/author.flow.spec.ts`, Playwright CI)
- [x] Search scenario: RAG answer with citations navigates to source document (`e2e/search.flow.spec.ts`, Playwright CI)
- [x] Admin scenario contract: backend source listing requires `knowledge.admin` (`scripts/smoke-knowledgebase-admin-ingest.test.mjs`; live probe optional via `SDKWORK_KNOWLEDGEBASE_SMOKE_BACKEND_URL`)
- [x] Open API scenario contract: api-key `context_packs` and `retrievals` (`scripts/smoke-knowledgebase-open-api.test.mjs`; live probe optional via `SDKWORK_KNOWLEDGEBASE_SMOKE_OPEN_URL`)
- [x] WeChat publish path blocks demo fallback in production builds (`shouldUseKnowledgebaseDemoFallback`; hosted API smoke optional before cutover)

### Operations

- [x] Postgres production path validated with `pnpm db:bootstrap`, `pnpm db:drift:check` (CI `database-postgres` job)
- [x] Backup/restore runbook documented (`deployments/runbooks/backup-restore.md`) and referenced by launch runbook
- [x] Split-services deployment smoke script: `SDKWORK_KNOWLEDGEBASE_SMOKE_BASE_URL=... pnpm test:smoke` (optional CI `staging-smoke` job)
- [x] JSON logging enabled in production topology; OTEL documented when collector is available (`SDKWORK_KNOWLEDGEBASE_LOG_FORMAT=json`)

### Release

- [x] Web bundle `web-production` manifest requires checksum/signature/SBOM per `sdkwork.app.config.json`
- [x] Three TypeScript SDK families indexed for release consumption (`specs/component.spec.json`)
- [x] Desktop packaging workflow explicitly prelaunch-disabled until desktop CI targets ship (`sdkwork.app.config.json` metadata)

Launch orchestration runbook: [deployments/runbooks/production-launch.md](../../deployments/runbooks/production-launch.md)

Automated launch gate:

```bash
pnpm test:launch-readiness
pnpm test:e2e:playwright
```

## Success Metrics (from PRD)

| Metric | Target |
|--------|--------|
| API availability (per tenant deployment) | 99.5% monthly |
| P95 retrieval latency (warm index) | < 2s |
| Authz failures return 403 without data leak | 100% in integration tests |
| PC shell smoke (login + load) | Pass in CI Playwright |
| PC author/search launch flows | Pass in CI Playwright |
| Document save success when online | > 99% |

## Out of Scope for 1.0

- Multi-tenant SaaS billing — see [PRD-phase2-commercial-saas.md](PRD-phase2-commercial-saas.md)
- Real-time collaborative editing
- Mobile native clients
- SOC2 program (platform-level)

## Database migration authority

Canonical lifecycle assets live under `database/`. The repository crate still embeds SQLite bootstrap SQL mirrored from historical migrations; do not add new schema files under `crates/.../migrations/` — use `database/migrations/{engine}/` and `pnpm db:materialize:contract`.
