# Tenant Isolation Operations Runbook

Status: active
Owner: SDKWork Knowledgebase operators

## Supported Model

Production uses one tenant and one organization per API/worker deployment. Every deployment binds:

- `SDKWORK_KNOWLEDGEBASE_TENANT_ID`
- `SDKWORK_KNOWLEDGEBASE_ORGANIZATION_ID` (required and non-zero in production-like environments)

PostgreSQL pools inject `app.current_tenant_id` and `app.current_organization_id` through
connection options before the shared process pool is created. RLS policies and ordinary
repository operations bind both columns. Request-scoped identity switching is unsupported;
never route a different tenant or organization through an existing deployment.
See [ADR-20260731-dedicated-tenant-organization-runtime.md](../architecture/decisions/ADR-20260731-dedicated-tenant-organization-runtime.md).

## Cross-Tenant Access Suspicion

1. Stop routing new traffic to the affected deployment.
2. Capture `traceId`, request time, actor, tenant, organization, and resource identifiers.
3. Search API and audit logs for `tenant_id_mismatch` or `organization_id_mismatch`.
4. Verify tenant and organization environment values on every pod.
5. Confirm backend requests use organization login scope and required permissions.
6. Query `kb_audit_event` and immutable platform audit records for the affected resource.
7. Rotate affected credentials and preserve database/log evidence before remediation.

## Safe Tenant Cutover

1. Provision a new dedicated deployment with reviewed tenant and organization values.
2. Restore or migrate data with approved backup/restore tooling.
3. Run PostgreSQL isolation, readiness, API smoke, and rollback checks.
4. Switch routing only after evidence is recorded.
5. Revoke old deployment secrets before decommissioning it.

## Verification

- `integration_tenant_isolation` passes in CI.
- A request with the wrong tenant or organization returns HTTP 403 ProblemDetail.
- Release PostgreSQL verifies forced RLS with a non-owner application role.
- Request-scoped shared-pool mode is outside the supported production architecture.
