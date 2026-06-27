# SDKWork Knowledgebase — Phase 2 Commercial SaaS

Status: **in progress** — Phase 2.0 foundations and Phase 2.1 RLS migration complete; Phase 2.2 connection checkout wired  
Owner: SDKWork maintainers  
Application: sdkwork-knowledgebase  
Updated: 2026-06-25  
Parent: [PRD.md](PRD.md)  
Prerequisite: [PRD-mvp-launch.md](PRD-mvp-launch.md) Phase 1.0 complete

## Purpose

Define commercial SaaS landing criteria beyond Phase 1.0 single-tenant production launch. Phase 1.0 delivers **tenant-dedicated deployments**; Phase 2 delivers **shared multi-tenant subscription product**.

## Phase 1.0 vs Phase 2

| Capability | Phase 1.0 (production launch) | Phase 2 (commercial SaaS) |
| --- | --- | --- |
| Deployment model | One tenant per API/worker process | Shared multi-tenant platform |
| Billing | Platform-level or manual | Seat/usage metering (Stripe or SDKWork platform) |
| Auth | IAM dual-token + backend `knowledge.admin` | + SSO/SCIM (platform IAM) |
| Data isolation | App-layer `tenant_id` filters | Postgres RLS ([ADR decided](../../adr/ADR-2026-06-24-phase2-postgres-rls-multi-tenant.md); migration Phase 2.1) |
| Desktop | Prelaunch-disabled | Enabled with CI packaging |
| Legal | AGPL review for redistribution | Commercial license or SaaS exception |

## Phase 2 exit criteria

### Platform

- [x] Multi-tenant isolation **decision** documented — [ADR-2026-06-24-phase2-postgres-rls-multi-tenant.md](../../adr/ADR-2026-06-24-phase2-postgres-rls-multi-tenant.md)
- [x] Postgres RLS migration shipped — `database/migrations/postgres/0007_knowledgebase_postgres_rls.up.sql`
- [x] Connection checkout sets `app.current_tenant_id` on every pooled Postgres connection (Phase 2.2)
- [x] Usage metering exported for billing (Prometheus counters + structured `billing_event` JSON logs)
- [ ] Per-tenant quota: API rate tiers, storage, ingest concurrency, retrieval QPS
- [ ] Tenant self-service signup and subscription lifecycle

### Product

- [ ] Admin console for tenant operators (sources, indexes, members) without raw backend API
- [x] Audit retention policy documented ([audit-retention.md](../../runbooks/audit-retention.md))
- [ ] GDPR export/delete workflows automated for `kb_audit_event`
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
# Phase 2.1 additions (planned):
# pnpm test:multi-tenant-isolation
# pnpm test:billing-metering
```

## References

- [ADR-2026-06-24-phase2-postgres-rls-multi-tenant.md](../../adr/ADR-2026-06-24-phase2-postgres-rls-multi-tenant.md)
- [PRD.md](PRD.md) §7 Phases
- [docs/runbooks/tenant-isolation.md](../../runbooks/tenant-isolation.md)
- `../sdkwork-specs/IAM_SPEC.md`, `SECURITY_SPEC.md`, `RELEASE_SPEC.md`
