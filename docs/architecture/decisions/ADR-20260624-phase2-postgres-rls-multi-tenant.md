# ADR-20260624-phase2-postgres-rls-multi-tenant

Status: deprecated
Owner: SDKWork Knowledgebase maintainers
Date: 2026-06-24
Specs: ARCHITECTURE_DECISION_SPEC.md, DATABASE_SPEC.md, SECURITY_SPEC.md, RELEASE_SPEC.md
Related: [ADR-20260731-dedicated-tenant-organization-runtime.md](ADR-20260731-dedicated-tenant-organization-runtime.md)

## Context

This record captured an earlier shared-process roadmap. It is not the release-candidate
architecture: it omitted organization-scoped RLS and depended on request-scoped connection
checkout that was never implemented or approved for production.

All knowledge tables already carry `tenant_id` columns and composite indexes. Integration tests enforce cross-tenant access denial at the HTTP layer.

## Decision

The retained part of this decision is the use of PostgreSQL RLS as defense in depth. The
tenant-only policy and shared-process roadmap are deprecated. The proposed replacement binds both
tenant and organization to a dedicated deployment.

## Alternatives

- **Schema-per-tenant:** higher operational cost, slower tenant onboarding, harder migrations at scale.
- **Application-only filters:** insufficient for shared-process SaaS; one query bug could become a cross-tenant data breach.

## Deprecated Implementation Plan

### Phase 2.1: RLS policies (database)

1. Enable RLS on all `kb_*` tables and tenant-scoped views.
2. Create policy `tenant_isolation` using session variable `app.current_tenant_id` set at connection checkout.
3. Ship migration under `database/migrations/postgres/`; mirror contract via `pnpm db:materialize:contract`.

### Phase 2.2: Connection tenant context (runtime)

1. Set `SET app.current_tenant_id = $tenant` on every pooled connection after auth resolution.
2. Fail closed when tenant context is missing in production-like environments.

### Phase 2.3: Billing and quotas

1. Prometheus counters: `knowledge_retrievals_total`, `knowledge_context_packs_total`, `knowledge_ingest_jobs_*` (implemented).
2. Structured JSON billing events (`billing_event=*`) for log pipeline aggregation.
3. Per-tenant rate tiers via existing Redis rate limit store with tenant-scoped keys.

## Consequences

- Positive: defense in depth, auditable isolation, aligns with SDKWork IAM tenant model.
- Negative: migration complexity, connection pools must set session context reliably.
- Neutral: this record provides decision history only and must not drive runtime configuration.

## Verification

```bash
pnpm verify
pnpm test:phase2-readiness
pnpm test:multi-tenant-isolation
```

## Supersedes / Superseded By

Proposed replacement:
[ADR-20260731-dedicated-tenant-organization-runtime.md](ADR-20260731-dedicated-tenant-organization-runtime.md).
The retired `docs/adr/` compatibility path continues to redirect to this historical record.
