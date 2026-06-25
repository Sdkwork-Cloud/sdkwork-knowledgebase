# ADR-2026-06-24: Phase 2 Multi-Tenant Postgres Isolation

Status: accepted  
Date: 2026-06-24  
Deciders: SDKWork Knowledgebase maintainers  
Related: [PRD-phase2-commercial-saas.md](../product/prd/PRD-phase2-commercial-saas.md)

## Context

Phase 1.0 binds one tenant per API/worker deployment with application-layer `tenant_id` filters on every query. Phase 2 requires a shared multi-tenant SaaS platform where many tenants share one Postgres cluster and process fleet.

All knowledge tables already carry `tenant_id` columns and composite indexes. Integration tests enforce cross-tenant access denial at the HTTP layer.

## Decision

Adopt **Postgres Row Level Security (RLS)** on tenant-scoped tables, retaining existing `tenant_id` columns and application guards as defense in depth.

Rejected alternatives:

- **Schema-per-tenant:** higher operational cost, slower tenant onboarding, harder migrations at scale.
- **Application-only filters:** insufficient for shared-process SaaS; one ORM bug becomes a data breach.

## Implementation phases

### Phase 2.1 — RLS policies (database)

1. Enable RLS on all `kb_*` tables and tenant-scoped views.
2. Create policy `tenant_isolation` using session variable `app.current_tenant_id` set at connection checkout.
3. Ship migration under `database/migrations/postgres/`; mirror contract via `pnpm db:materialize:contract`.

### Phase 2.2 — Connection tenant context (runtime)

1. Set `SET app.current_tenant_id = $tenant` on every pooled connection after auth resolution.
2. Fail closed when tenant context is missing in production-like environments.

### Phase 2.3 — Billing and quotas

1. Prometheus counters: `knowledge_retrievals_total`, `knowledge_context_packs_total`, `knowledge_ingest_jobs_*` (implemented).
2. Structured JSON billing events (`billing_event=*`) for log pipeline aggregation.
3. Per-tenant rate tiers via existing Redis rate limit store with tenant-scoped keys.

## Consequences

- Positive: defense in depth, auditable isolation, aligns with SDKWork IAM tenant model.
- Negative: migration complexity, connection pool must set session context reliably.
- Neutral: Phase 1.0 single-tenant deployments continue unchanged until RLS migration is applied.

## Verification

```bash
pnpm verify
pnpm test:phase2-readiness
# Future: pnpm test:multi-tenant-isolation (RLS integration tests)
```
