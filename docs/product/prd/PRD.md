# SDKWork Knowledgebase PRD

Status: active
Owner: SDKWork maintainers
Application: sdkwork-knowledgebase
Updated: 2026-06-24
Specs: REQUIREMENTS_SPEC.md, DOCUMENTATION_SPEC.md

## Document Map

- [PRD-mvp-launch.md](PRD-mvp-launch.md) — MVP launch scope and acceptance criteria
- [PRD-phase2-commercial-saas.md](PRD-phase2-commercial-saas.md) — Phase 2 multi-tenant commercial SaaS criteria

## 1. Background And Problem

Teams need a knowledge platform that combines structured documentation, retrieval-augmented search, and AI-assisted authoring without sacrificing tenant isolation, auditability, or SDKWork platform integration. Existing wikis often lack native RAG, OKF knowledge bundles, and consistent IAM/SDK contracts.

## 2. Target Users

| Persona | Need |
|---------|------|
| Knowledge author | Create/edit documents, import from Drive/Git, publish to WeChat or public sites |
| Team member | Search, read, collaborate within granted spaces |
| Tenant admin | Manage spaces, members, ingestion sources, OKF profiles |
| Platform operator | Operate backend-api, worker, observability, and per-tenant deployments |
| Integrator / ISV | Consume Open API and generated SDKs for retrieval and ingest |

## 3. Goals And Non-Goals

**Goals (MVP)**

- Secure multi-surface API (app / backend / open) with SDK-first clients
- Knowledge spaces with Drive-backed storage, ingest, and RAG retrieval
- PC browser and desktop shell for authoring, search, and market/catalog
- Production deployment topology (split services, worker, Postgres, probes, audit)
- Fail-closed auth, tenant/org guards, and backend `knowledge.admin` RBAC

**Non-Goals (MVP)**

- Shared multi-tenant SaaS billing (Stripe/seat metering) — delegated to SDKWork platform or Phase 2
- Real-time collaborative editing
- Mobile native clients
- Full enterprise compliance program (SOC2) — platform-level concern

## 4. Scope

**In scope**

- App API: spaces, documents, browser, ingest, retrieval, agent chat, WeChat, market, site deploy
- Backend API: sources, OKF compile/candidates, indexes, retrieval profiles/traces
- Open API: retrieval, context packs, ingest, document/browser read
- Worker: outbox dispatch, ingestion maintenance
- PC client: editor, search, settings, offline/network awareness

**Out of scope (MVP)**

- Payment/subscription product billing
- SCIM/SAML beyond platform IAM
- Page-level real-time co-editing

## 5. User Scenarios

1. **Author creates a space and document** — login → create space → create doc → edit in TipTap → auto-save via SDK
2. **Member searches knowledge** — search module → RAG answer with citations → navigate to source doc
3. **Admin configures ingest** — backend operator with `knowledge.admin` → create source → worker completes job → retrieval returns new chunks
4. **Integrator retrieves via Open API** — API key with org context → retrieval → context pack for downstream agent
5. **Operator deploys tenant** — K8s manifests + env (tenant, org, secrets, outbox webhook) → `/readyz` green → smoke test

## 6. Success Metrics

| Metric | MVP target |
|--------|------------|
| API availability (per tenant deployment) | 99.5% monthly |
| P95 retrieval latency (warm index) | < 2s |
| Authz failures correctly return 403 (no data leak) | 100% in integration tests |
| Critical security alignment tests | Pass in CI |
| PC shell smoke (login + load) | Pass in CI Playwright |
| Document save success when online | > 99% |

## 7. Phases

| Phase | Focus | Exit criteria |
|-------|-------|---------------|
| **1.0 (current)** | Production launch (single-tenant-per-process) | Postgres prod path, runbooks, PRD acceptance, E2E on real API |
| **2.0** | Commercial SaaS | Shared multi-tenant, billing, quotas, GDPR workflows |
| **3.0** | Industry parity | Real-time collab, analytics, mobile |

## 8. Linked Requirements

- `docs/architecture/tech/TECH-2026-06-01-knowledgebase-backend-design.md`
- `docs/architecture/tech/TECH-2026-06-09-knowledgebase-agent-rag-design.md`
- `../sdkwork-specs/SECURITY_SPEC.md`, `IAM_SPEC.md`, `APP_SDK_INTEGRATION_SPEC.md`
- `deployments/README.md` — tenant isolation and observability

## 9. Resolved And Open Questions

- **Multi-tenant data model:** Postgres RLS — decided in [ADR-2026-06-24-phase2-postgres-rls-multi-tenant.md](../adr/ADR-2026-06-24-phase2-postgres-rls-multi-tenant.md); migration shipped for Phase 2.1
- **Billing owner:** SDKWork platform vs standalone Stripe — open; decide before Phase 2 commercial launch
- **Minimum enterprise audit retention period:** documented in [audit-retention.md](../runbooks/audit-retention.md); automated purge/export jobs remain Phase 2.4
