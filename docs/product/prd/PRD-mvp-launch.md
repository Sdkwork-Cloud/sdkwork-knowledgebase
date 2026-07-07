# SDKWork Knowledgebase — MVP Launch Acceptance

Status: prelaunch-gated
Owner: SDKWork maintainers
Application: sdkwork-knowledgebase
Updated: 2026-07-07
Parent: [PRD.md](PRD.md)

## Purpose

Phase 0.1 exit criteria and Phase 1.0 launch acceptance checklist for SDKWork Knowledgebase. This document records repository readiness gates and remaining release blockers; it is not a production release evidence record.

## Commercialization Readiness Decision

Decision: SDKWork Knowledgebase remains prelaunch and must not be treated as a production/commercial release until release-governance evidence is attached. The app manifest now blocks publication through `publish.status=INACTIVE`, `release.defaultChannel=DEV`, disabled prelaunch packages, and disabled placeholder media projection.

- [x] Align manifest launch state: `sdkwork.app.config.json` now projects `publish.status=INACTIVE`, `release.defaultChannel=DEV`, `release.latest.DEV=0.1.0`, and `metadata.releaseStatus=prelaunch-gated`.
- [ ] Replace placeholder catalog media: icon, screenshot, and preview entries are disabled with `generatedPlaceholder=true` and `releaseStatus=prelaunch-placeholder`; production listing requires Drive-backed, real product media assets.
- [ ] Attach `web-universal-cloud-browser-zip` artifact evidence: checksum value, signing evidence, SBOM, provenance/attestation, immutable artifact URL or digest, and build workflow run.
- [ ] Record rollout, rollback, monitoring, and smoke-test evidence for each runtime target and deployment profile.
- [ ] Run and record release-environment PostgreSQL verification with `SDKWORK_KNOWLEDGEBASE_DATABASE_URL` pointing at the target PostgreSQL service; local SQLite and contract gates are not enough for a commercial cutover claim.
- [ ] Run and archive final launch gates on the release candidate artifact: `pnpm verify`, `pnpm test`, `pnpm test:e2e:playwright`, and live smoke probes with configured app/backend/open API URLs.

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
- [x] WeChat credentials encrypted at rest; `encrypt_secret` fails closed when `SDKWORK_KNOWLEDGEBASE_SECRETS_ENCRYPTION_KEY` is unset (no plaintext fallback)

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
- [x] Offline import modals (chat file/dialog, notes) show honest empty states when IM connector is not wired; batch import remains gated by `assertKnowledgebasePreviewFeature`
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
- [x] Cloud drive import modal uses cursor pagination (`listBrowserItemsPage`) with Load more on my-drive browse
- [x] Cloud drive starred/recent/shared collections paginate through Drive `pageToken` (capped at 500 items)
- [x] Drive import pipeline enforces `MAX_MARKDOWN_PAYLOAD_BYTES` before chunking
- [x] WeChat typography preview uses article author and current date instead of hardcoded demo metadata
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
- [x] WeChat publish modal uses API-backed account selection; fan tag groups load from WeChat `tags/get` API via `wechat.officialAccounts.fanTags.list`; mass send uses `message/mass/sendall` when `sendNotification` is enabled
- [x] WeChat publish path blocks demo fallback in production builds (`shouldUseKnowledgebaseDemoFallback`; hosted API smoke optional before cutover)

### Operations

- [x] PostgreSQL lifecycle path covered by CI/database gates; release-environment PostgreSQL verification with the target service remains a cutover blocker above.
- [x] Backup/restore runbook documented (`deployments/runbooks/backup-restore.md`) and referenced by launch runbook
- [x] Split-services deployment smoke script: `SDKWORK_KNOWLEDGEBASE_SMOKE_BASE_URL=... pnpm test:smoke` (optional CI `staging-smoke` job)
- [x] JSON logging enabled in production topology; OTEL documented when collector is available (`SDKWORK_KNOWLEDGEBASE_LOG_FORMAT=json`)

### Release

- [ ] Web bundle `web-universal-cloud-browser-zip` release evidence attached: checksum, signature, SBOM, provenance/attestation, immutable artifact reference, and workflow run. The manifest declares these controls as required and keeps the package disabled until evidence exists.
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
