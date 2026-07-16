# SDKWork Knowledgebase PRD

Status: active
Owner: SDKWork maintainers
Application: sdkwork-knowledgebase
Updated: 2026-07-14
Specs: REQUIREMENTS_SPEC.md, DOCUMENTATION_SPEC.md

## Document Map

- [PRD-mvp-launch.md](PRD-mvp-launch.md) - MVP launch scope and acceptance criteria
- [PRD-phase2-commercial-saas.md](PRD-phase2-commercial-saas.md) - Phase 2 multi-tenant commercial SaaS criteria

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
- Production deployment topology (replicated application public ingress, worker, Postgres, probes, audit)
- Fail-closed auth, tenant/org guards, and backend `knowledge.admin` RBAC
- Managed IM Conversation group knowledge spaces that are created only by the current IM Owner's
  first initialization or failed-provisioning retry, use dedicated group binding and
  current-membership authorization, and open in the full standalone browser or Tauri Knowledgebase
  application only for active joined non-Guest Owners, Admins, and Members. See
  [REQ-2026-0713-group-knowledgebase.md](../requirements/REQ-2026-0713-group-knowledgebase.md).

**Non-Goals (MVP)**

- Shared multi-tenant SaaS billing (Stripe/seat metering) - delegated to SDKWork platform or Phase 2
- Real-time collaborative editing
- Mobile native clients
- Full enterprise compliance program (SOC2) - platform-level concern

## 4. Scope

**In scope**

- App API: spaces, documents, browser, ingest, retrieval, agent chat, WeChat, market, site deploy
- Provider-backed media tasks: ClawRouter SDK image generation and speech-to-text; requests fail closed when provider configuration is absent
- Drive-backed static site publishing: enabled only when an HTTPS public object gateway is configured; unsupported third-party hosting is not reported as successful
- Backend API: sources, OKF compile/candidates, indexes, retrieval profiles/traces
- Open API: retrieval, context packs, ingest, document/browser read
- Worker: outbox dispatch, ingestion maintenance
- PC client: editor, search, settings, offline/network awareness
- OKF original file list: knowledgebase file lists call `spaces.browser.list?view=files`; for OKF spaces this displays original source files under `sources/raw` only and must not expose `okf/`, `output/`, `.sdkwork/`, or Drive root system folders.
- OKF browser view separation: OKF concept and bundle tooling uses `view=okf_bundle`; generated output tooling uses `view=outputs`. Root uploads and root folder creation use response `data.parentId` as the Drive parent folder id, never Drive root or a hard-coded `sources/raw` path.

**Out of scope (MVP)**

- Payment/subscription product billing
- SCIM/SAML beyond platform IAM
- Page-level real-time co-editing

## 5. User Scenarios

1. **Author creates a space and document** - login -> create space -> create doc -> edit in TipTap -> auto-save via SDK
2. **Member searches knowledge** - search module -> RAG answer with citations -> navigate to source doc
3. **Admin configures ingest** - backend operator with `knowledge.admin` -> create source -> worker completes job -> retrieval returns new chunks
4. **Integrator retrieves via Open API** - API key with org context -> retrieval -> context pack for downstream agent
5. **Operator deploys tenant** - K8s manifests + env (tenant, org, secrets, outbox webhook) -> `/readyz` green -> smoke test

6. **Author manages OKF source files** - open OKF space file list -> view original files from `sources/raw` through `view=files` -> upload or create root folder using `data.parentId` -> inspect generated OKF concepts only through `view=okf_bundle`
7. **Group member opens managed knowledge** - the current IM Owner initializes the group action or
   retries failed provisioning -> exactly one managed space is provisioned -> after activation,
   joined non-Guest Owner, Admin, and Member roles open only that fixed workspace through a one-time
   ticket -> Guest, left, removed, and non-member actors are denied -> removal, role reduction, or
   dissolution updates access and archive state without deleting documents by default

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
- `docs/product/requirements/REQ-2026-0713-group-knowledgebase.md`
- `docs/architecture/decisions/ADR-20260713-group-knowledgebase-binding-and-launch.md`
- `specs/okf-knowledge-bundle.spec.json` - OKF bundle layers, browser views, and raw source file list contract
- `../sdkwork-specs/SECURITY_SPEC.md`, `IAM_SPEC.md`, `APP_SDK_INTEGRATION_SPEC.md`
- `deployments/README.md` - tenant isolation and observability

## 9. Resolved And Open Questions

- **Multi-tenant data model:** Postgres RLS - decided in [ADR-20260624-phase2-postgres-rls-multi-tenant.md](../../architecture/decisions/ADR-20260624-phase2-postgres-rls-multi-tenant.md); migration shipped for Phase 2.1
- **Billing owner:** SDKWork platform vs standalone Stripe - open; decide before Phase 2 commercial launch
- **Minimum enterprise audit retention period:** documented in [audit-retention.md](../runbooks/audit-retention.md); automated purge/export jobs remain Phase 2.4
