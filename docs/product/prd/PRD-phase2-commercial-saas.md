# SDKWork Knowledgebase — Phase 2 Commercial SaaS

Status: **in progress** — Phase 2.0 foundations and Phase 2.1 RLS migration complete; Phase 2.2 connection checkout wired  
Owner: SDKWork maintainers  
Application: sdkwork-knowledgebase  
Updated: 2026-07-05  
Parent: [PRD.md](PRD.md)  
Prerequisite: [PRD-mvp-launch.md](PRD-mvp-launch.md) Phase 1.0 complete

## Pre-launch security hardening (2026-07-04)

The following P0 items are implemented before first production cutover:

- Upload session create/complete enforces `require_space_access`
- Space ACL fail-closes when `drive_space_id` is missing
- WeChat editor HTML sanitized via `sanitizeEditorHtml`
- Production demo/synthetic media gated by `shouldUseKnowledgebaseDemoFallback()`
- Backend admin scope narrowed to `knowledge.platform.manage`, `knowledge.admin`, and `knowledge.*`
- Tenant-scoped dynamic rate limit / CORS / tenant profile policies wired from `web_rate_limit_policy` via `SqlxDynamicPolicyBundle` when `SDKWORK_WEB_STORE_DATABASE_URL` is configured

Verification:

```bash
pnpm test:security
pnpm test:multi-tenant-isolation
```

## Purpose

Define commercial SaaS landing criteria beyond Phase 1.0 single-tenant production launch. Phase 1.0 delivers **tenant-dedicated deployments**; Phase 2 delivers **shared multi-tenant subscription product**.

## Phase 1.0 vs Phase 2

| Capability | Phase 1.0 (production launch) | Phase 2 (commercial SaaS) |
| --- | --- | --- |
| Deployment model | One tenant per API/worker process | Shared multi-tenant platform |
| Billing | Platform-level or manual | Seat/usage metering (Stripe or SDKWork platform) |
| Auth | IAM dual-token + backend `knowledge.admin` | + SSO/SCIM (platform IAM) |
| Data isolation | App-layer `tenant_id` filters | Postgres RLS ([ADR decided](../../architecture/decisions/ADR-20260624-phase2-postgres-rls-multi-tenant.md); migration Phase 2.1) |
| Desktop | Prelaunch-disabled | Enabled with CI packaging |
| Legal | AGPL review for redistribution | Commercial license or SaaS exception |

## Phase 2 exit criteria

### Platform

- [x] Multi-tenant isolation **decision** documented — [ADR-20260624-phase2-postgres-rls-multi-tenant.md](../../architecture/decisions/ADR-20260624-phase2-postgres-rls-multi-tenant.md)
- [x] Postgres RLS migration shipped — `database/migrations/postgres/0007_knowledgebase_postgres_rls.up.sql`
- [x] Connection checkout sets `app.current_tenant_id` on every pooled Postgres connection (Phase 2.2)
- [x] Usage metering exported for billing (Prometheus counters + structured `billing_event` JSON logs)
- [x] Tenant status API: `GET /backend/v3/api/knowledge/tenants/current` implemented in `HostedBackendApi`
- [x] Tenant isolation spec documented — [specs/tenant-isolation.md](../../../specs/tenant-isolation.md)
- [x] Per-tenant quota: API rate tiers wired from `web_rate_limit_policy` + Redis counters; **business quotas enforced** for document count, ingest concurrency, retrieval rate per minute, and **storage bytes** (`SUM(kb_drive_object_ref.size_bytes)` vs `SDKWORK_KNOWLEDGEBASE_TENANT_MAX_STORAGE_BYTES`, default 100 GiB) on markdown ingest, upload session complete, and drive import; quota status exposed on `GET /backend/v3/api/knowledge/tenants/current` (`KnowledgeTenantStatus.quota`) and in `/admin` console
- [ ] Tenant self-service signup and subscription lifecycle

### Product

- [x] Admin console for tenant operators without raw backend API — **`/admin` console** (tenant status, spaces, members, sources, indexes, retrieval traces, provider health via backend SDK)
- [x] Dev/staging bootstrap seeds default `web_rate_limit_policy` tiers and `web_tenant_runtime_profile` when web store DB is configured
- [x] Audit retention policy documented ([audit-retention.md](../../runbooks/audit-retention.md))
- [x] GDPR export/delete workflows automated for `kb_audit_event` via backend compliance API (`compliance.auditEvents.export`, `compliance.auditEvents.anonymizeActor`)
- [ ] SLA dashboard: availability, P95 retrieval, error budget alerts

### Release

- [ ] SDK families published to registry (`releaseState: published`)
- [ ] Desktop bundles enabled in `sdkwork.app.config.json` with signed CI artifacts
- [ ] AGPL/commercial licensing decision documented and enforced in supply chain

## Verification (Phase 2)

Phase 2 adds gates on top of Phase 1.0:

```bash
pnpm verify
pnpm test:launch-readiness
pnpm test:phase2-readiness
# Phase 2.1 additions:

pnpm test:multi-tenant-isolation
pnpm test:billing-metering
pnpm test:tenant-quota
```

## References

- [ADR-20260624-phase2-postgres-rls-multi-tenant.md](../../architecture/decisions/ADR-20260624-phase2-postgres-rls-multi-tenant.md)
- [PRD.md](PRD.md) §7 Phases
- [docs/runbooks/tenant-isolation.md](../../runbooks/tenant-isolation.md)
- [specs/tenant-isolation.md](../../../specs/tenant-isolation.md) — Tenant isolation architecture specification
- `../sdkwork-specs/IAM_SPEC.md`, `SECURITY_SPEC.md`, `RELEASE_SPEC.md`
