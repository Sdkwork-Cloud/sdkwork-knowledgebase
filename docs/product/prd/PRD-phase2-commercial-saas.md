# SDKWork Knowledgebase Phase 2 Commercial SaaS

Status: **blocked / prelaunch**
Owner: SDKWork maintainers
Application: `sdkwork-knowledgebase`
Updated: 2026-07-14
Parent: [PRD.md](PRD.md)
Prerequisite: [PRD-mvp-launch.md](PRD-mvp-launch.md)

## Purpose

Phase 1 supports a tenant-dedicated API and worker deployment. Phase 2 is a shared
multi-tenant subscription product. Database policies and static gates alone do not
prove that shared SaaS is safe or commercially releasable.

## Proven Foundations

- App, backend, and open APIs use authenticated SDKWork request contexts and fail-closed tenant guards.
- Upload session create/complete enforces space access.
- Missing Drive bindings fail closed for space ACL checks.
- AI and WeChat services fail closed when their app SDK capability is unavailable.
- PostgreSQL RLS policy migrations exist for tenant-scoped tables.
- Deployment-dedicated PostgreSQL pools bind `app.current_tenant_id` from deployment configuration.
- Billing counters, structured billing events, tenant status, and audit compliance APIs exist.
- Document count, atomic ingest concurrency, retrieval rate, and measured upload/object storage quotas exist.
- Backend admin list endpoints use bounded store-level cursor pagination.

## Exit Criteria

### Platform

- [x] Multi-tenant isolation decision and tenant-isolation specification are documented.
- [x] PostgreSQL RLS policy migration is materialized.
- [ ] Shared request checkout uses a transaction-scoped `SET LOCAL app.current_tenant_id` for every tenant repository operation.
- [ ] PostgreSQL tests prove isolation when pooled connections are reused, rolled back, cancelled, and reassigned between two tenants.
- [ ] Drive import reserves and checks projected object bytes before enqueueing.
- [ ] A crashed ingestion worker can safely recover a claimed job through a reviewed lease/token schema.
- [x] Usage metering and tenant status are exposed to operators.
- [ ] Tenant signup, subscription, entitlement, suspension, and cancellation lifecycle is implemented.

### Operations

- [ ] Release PostgreSQL verification is recorded with production-equivalent TLS and credentials.
- [ ] Load, soak, fault-injection, and OOM/capacity tests establish supported limits.
- [ ] Prometheus Adapter recording rules are implemented before enabling request-rate or backlog HPA metrics.
- [ ] Public provider egress is constrained by a reviewed egress gateway or CNI FQDN policy.
- [ ] Availability, P95 latency, saturation, queue age, and error-budget alerts are operational.

### Release And Legal

- [ ] SDK families are published through release governance.
- [ ] Desktop artifacts are signed and include checksum, SBOM, and provenance evidence.
- [ ] Rollout, rollback, backup/restore, and live smoke evidence is attached to the release candidate.
- [ ] AGPL/commercial licensing and redistribution policy is approved and enforced.

## Supported Deployment Until Exit

Use one tenant per API/worker deployment or a dedicated database/schema. The existing
`after_connect` tenant setting is deployment-bound and must not be described as shared
request checkout. `sdkwork.app.config.json` must remain prelaunch-gated until every
release criterion above and the Phase 1 launch gates have evidence.

## Verification

```bash
pnpm check
pnpm test:security
pnpm test:phase2-readiness
pnpm test:multi-tenant-isolation
pnpm test:tenant-quota
pnpm test:billing-metering
pnpm api:materialize:check
pnpm sdk:generate:check
```

These static and local checks are necessary but do not replace release PostgreSQL,
capacity, security, and production operations evidence.

## References

- [ADR-20260624-phase2-postgres-rls-multi-tenant.md](../../architecture/decisions/ADR-20260624-phase2-postgres-rls-multi-tenant.md)
- [tenant-isolation.md](../../../specs/tenant-isolation.md)
- [tenant isolation runbook](../../runbooks/tenant-isolation.md)
- [PRD-mvp-launch.md](PRD-mvp-launch.md)
