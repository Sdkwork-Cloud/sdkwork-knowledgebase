# Tenant Isolation Operations Runbook

Status: active  
Owner: SDKWork Knowledgebase operators

## Model

Knowledgebase Phase 1.0 uses **single-tenant-per-process**. Each deployment binds:

- `SDKWORK_KNOWLEDGEBASE_TENANT_ID`
- `SDKWORK_KNOWLEDGEBASE_ORGANIZATION_ID` (non-zero in production-like environments)

Phase 2 shared SaaS adds Postgres RLS policies — see [ADR-2026-06-24-phase2-postgres-rls-multi-tenant.md](../adr/ADR-2026-06-24-phase2-postgres-rls-multi-tenant.md).

## Incident: cross-tenant access suspicion

1. Capture `x-request-id` from the report.
2. Search API logs for `tenant_id_mismatch` or `organization_id_mismatch`.
3. Verify deployment env on every pod: tenant and organization must match manifest.
4. Confirm backend-api requests use `login_scope = ORGANIZATION`.
5. Query `kb_audit_event` for the affected resource and actor.

## Safe tenant cutover

1. Provision a new deployment with the target tenant/org env.
2. Migrate data with approved backup/restore tooling.
3. Switch DNS/gateway routing only after `/readyz` and smoke tests pass.
4. Decommission the old deployment secrets.

## Verification

- Integration tests `integration_tenant_isolation` remain green in CI.
- Manual Open API call with wrong tenant context returns `403`.
