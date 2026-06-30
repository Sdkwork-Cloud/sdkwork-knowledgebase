# SDKWork Knowledgebase Technical Architecture

Status: active  
Owner: SDKWork maintainers  
Updated: 2026-06-29  
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

## 1. Architecture Overview

SDKWork Knowledgebase is a split-services Rust backend with a PC React client (browser + optional Tauri desktop). Each production deployment binds **one tenant per API/worker process** with fail-closed tenant and organization guards.

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
- **Memory**: `sdkwork-memory` via `sdkwork-knowledgebase-memory` port only
- **Frontend**: React 19, Vite, TipTap, IAM app SDK, generated knowledgebase app SDK, `@sdkwork/drive-app-sdk` for persistent uploads
- **Client composition**: native authority per `APP_COMPOSITION_SPEC.md` — root `pnpm-workspace.yaml`, pc-core `sdkDependencies`, and capability packages import SDK types only via `sdkwork-knowledgebase-pc-core/sdk`
- **Observability**: Prometheus `/metrics` (in-cluster only), structured audit logs, optional OTLP

## 3. System Boundaries

- Business logic: `sdkwork-knowledgebase-service`
- Persistence: `sdkwork-knowledgebase-repository-sqlx` + `database/` lifecycle
- HTTP boundaries: `sdkwork-routes-knowledgebase-{app,backend,open}-api`
- Background work: `sdkwork-knowledgebase-worker` (outbox + ingest maintenance)
- PC client: `apps/sdkwork-knowledgebase-pc/`

## 4. Security Model

- Production boot is fail-closed: Postgres, Redis rate limiting, secrets encryption, web audit persistence
- Backend OpenAPI declares `x-sdkwork-permission: knowledge.admin` on all protected operations
- Public ingress exposes API paths only; `/metrics` is scraped via ServiceMonitor inside the cluster
- PC production builds disable demo/mock API fallbacks

## 5. Deployment Topology

Production uses `cloud.split-services.production` with separate Deployments for app-api, backend-api, open-api, and worker. See `deployments/README.md` and `configs/topology/`.

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
