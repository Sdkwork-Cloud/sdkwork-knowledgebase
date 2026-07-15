# SDKWork Knowledgebase Technical Architecture

Status: active  
Owner: SDKWork maintainers  
Updated: 2026-07-14<br>
Specs: ARCHITECTURE_DECISION_SPEC.md, DOCUMENTATION_SPEC.md

## Document Map


- [TECH-alignment-baseline-2026-06-29.md](TECH-alignment-baseline-2026-06-29.md)
- [TECH-2026-06-11-sdkwork-structure-standardization-design.md](TECH-2026-06-11-sdkwork-structure-standardization-design.md)
- [TECH-2026-06-11-sdkwork-structure-standardization-implementation.md](TECH-2026-06-11-sdkwork-structure-standardization-implementation.md)
- [TECH-2026-06-19-okf-knowledge-bundle-design.md](TECH-2026-06-19-okf-knowledge-bundle-design.md)
- [TECH-okf-knowledge-bundle.md](TECH-okf-knowledge-bundle.md)
- [TECH-2026-06-01-knowledgebase-backend-design.md](TECH-2026-06-01-knowledgebase-backend-design.md)
- [TECH-2026-06-01-knowledgebase-backend-phase1-implementation.md](TECH-2026-06-01-knowledgebase-backend-phase1-implementation.md)
- [TECH-2026-06-09-knowledgebase-agent-rag-design.md](TECH-2026-06-09-knowledgebase-agent-rag-design.md)
- [TECH-2026-06-09-knowledgebase-agent-rag-implementation.md](TECH-2026-06-09-knowledgebase-agent-rag-implementation.md)
- [TECH-2026-06-12-knowledgebase-open-api-design.md](TECH-2026-06-12-knowledgebase-open-api-design.md)
- [TECH-2026-06-12-knowledgebase-open-api-implementation.md](TECH-2026-06-12-knowledgebase-open-api-implementation.md)
- [TECH-topology-standard.md](TECH-topology-standard.md)
- [PRD-mvp-launch.md](../../product/prd/PRD-mvp-launch.md)
- [ADR-20260624-phase2-postgres-rls-multi-tenant.md](../decisions/ADR-20260624-phase2-postgres-rls-multi-tenant.md)
- [ADR-20260713-group-knowledgebase-binding-and-launch.md](../decisions/ADR-20260713-group-knowledgebase-binding-and-launch.md)

## 1. Architecture Overview

SDKWork Knowledgebase is a Rust backend with separately deployable API and worker processes plus a PC React client (browser + optional Tauri desktop). Each production deployment binds **one tenant per API/worker process** with fail-closed tenant and organization guards.

| Surface | Prefix | SDK family | Auth |
|---------|--------|------------|------|
| App API | `/app/v3/api` | `sdkwork-knowledgebase-app-sdk` | dual-token |
| Backend API | `/backend/v3/api` | `sdkwork-knowledgebase-backend-sdk` | dual-token + `knowledge.admin` |
| Open API | `/knowledge/v3/api` | `sdkwork-knowledgebase-sdk` | API key |
| Worker | — | — | internal |

OpenAPI contracts are authored in `sdks/*/openapi/`, synchronized to `apis/` via `pnpm api:materialize`, and consumed by generated TypeScript SDKs.

## 2. Technology Choices

- **Backend**: Rust, Axum, SQLx, `sdkwork-web-framework`, PostgreSQL (production), SQLite (local dev)
- **Storage**: `sdkwork-drive` via `sdkwork-knowledgebase-drive` adapter only
- **OKF browser views**: PC file lists use `spaces.browser.list?view=files`, which resolves OKF spaces to `sources/raw` original files. OKF bundle inspection uses `view=okf_bundle`; generated outputs use `view=outputs`.
- **Memory**: `sdkwork-memory` via `sdkwork-knowledgebase-memory` port only
- **Frontend**: React 19, Vite, TipTap, IAM app SDK, generated knowledgebase app SDK, `@sdkwork/drive-app-sdk` for persistent uploads
- **Client pagination**: PC Cloud Drive browse/import uses generated Knowledgebase SDK and Drive SDK cursor page methods; interactive my-drive, starred, recent, and shared tabs load one page at a time and never prefetch multi-page aggregates
- **Client composition**: native authority per `APP_COMPOSITION_SPEC.md` — root `pnpm-workspace.yaml`, pc-core `sdkDependencies`, and capability packages import SDK types only via `sdkwork-knowledgebase-pc-core/sdk`
- **Observability**: Prometheus `/metrics` (in-cluster only), structured audit logs, optional OTLP

## 3. System Boundaries

- Business logic: `sdkwork-knowledgebase-service`
- Persistence: `sdkwork-knowledgebase-repository-sqlx` + `database/` lifecycle
- HTTP boundaries: `sdkwork-routes-knowledgebase-{app,backend,open}-api`
- Background work: `sdkwork-knowledgebase-worker` (outbox + ingest maintenance)
- Ingestion workers claim queued jobs with a conditional queued-to-running compare-and-set, so replicas cannot process the same job concurrently. Crash recovery leases are still a pre-launch schema item.
- Backend administrative list handlers use cursor page contracts and push ordering, filtering, and limits into database queries; full-list downloads are not a production path.
- PC client: `apps/sdkwork-knowledgebase-pc/`

## 4. Security Model

- Production boot is fail-closed: Postgres, Redis rate limiting, secrets encryption, web audit persistence
- Backend OpenAPI declares `x-sdkwork-permission: knowledge.admin` on all protected operations
- Public ingress exposes API paths only; `/metrics` is scraped via ServiceMonitor inside the cluster
- PC production builds disable demo/mock API fallbacks
- Managed group spaces use `kb_group_knowledge_space_binding` instead of generic context binding.
  The binding is scoped by tenant, organization, and IM Conversation id; group spaces are hidden
  from generic resource routes and resolved only through the specialized launch path.
- The group resolver requires both a synchronized IM role snapshot and direct Drive authorization.
  Current-Owner initialization and active-content access are separate: only the current IM Owner
  may initialize or retry failed provisioning. Once active, Owner maps to Owner, Admin to Writer,
  Member to Reader, and Guest to no access; left, removed, and non-member actors are also denied.
  ACL projection failure is fail-closed, and `active` binding state requires an active ACL
  projection.
- IM launch tickets are opaque, hash-stored, one-time, short-lived capabilities bound to verified
  actor/session scope, binding version, and membership epoch. Browser tickets are fragment-only;
  desktop tickets are transient deep-link data and never persistent host state.

## 4.1 Managed Group Knowledgebase Boundary

IM owns the Conversation roster and lifecycle. Knowledgebase owns the one-to-one managed binding,
space/Drive lifecycle, ACL projection, and final content enforcement. Trusted IM service calls use
the generated SDK/RPC boundary; the authenticated Knowledgebase App API consumes a ticket and
resolves the exact binding target. IM alone applies current-Owner initialization and retry
authorization before it requests provisioning; Knowledgebase never treats a browser-supplied role
as authority. It accepts launch tickets only after the binding is active and the interactive caller
is a joined non-Guest Owner, Admin, or Member. The browser opens the standalone `/group-launch`
route under its configured public base path. The desktop handoff uses the independent Knowledgebase
Tauri process, not an IM-owned iframe or Webview. See
[ADR-20260713-group-knowledgebase-binding-and-launch.md](../decisions/ADR-20260713-group-knowledgebase-binding-and-launch.md).

## 5. Deployment Topology

Production uses `cloud.production`; API and worker process decomposition is implementation detail inside that profile. The current Kubernetes descriptors run separate Deployments for app-api, backend-api, open-api, and worker. See `deployments/README.md` and `configs/topology/`.

## 6. Verification

```bash
pnpm check
pnpm check:app-composition
node ../sdkwork-specs/tools/check-api-response-envelope.mjs --workspace .
pnpm verify
pnpm test
```

Gates include architecture alignment, `verify-repo` native composition, PC app hygiene (SDK boundary), utils integration, API envelope, SDK generation, database contract, and Phase 1/2 readiness scripts.

Phase 1.0 launch acceptance: [PRD-mvp-launch.md](../../product/prd/PRD-mvp-launch.md).  
Phase 2 commercial SaaS: [PRD-phase2-commercial-saas.md](../../product/prd/PRD-phase2-commercial-saas.md).
