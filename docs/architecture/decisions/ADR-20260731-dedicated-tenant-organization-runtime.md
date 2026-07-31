# ADR-20260731-dedicated-tenant-organization-runtime

Status: proposed
Requirement: [PRD.md](../../product/prd/PRD.md)
Owner: SDKWork Knowledgebase maintainers
Date: 2026-07-31
Specs: ARCHITECTURE_DECISION_SPEC.md, DATABASE_SPEC.md, SECURITY_SPEC.md, DEPLOYMENT_SPEC.md, MIGRATION_SPEC.md

## Context

The release candidate has deployment-bound PostgreSQL pools and cannot safely change tenant or
organization scope during request checkout. The former shared-process roadmap described only a
tenant session variable and did not protect two organizations inside the same tenant. Treating
that roadmap as implemented would create a cross-organization disclosure risk.

Server SQLite also cannot supply the PostgreSQL RLS, locking, and cluster semantics required by the
production topology. It remains useful for bounded client-local state and deterministic tests, but
is not a server release profile.

## Decision

- The supported production unit is one dedicated API/worker deployment bound to one canonical
  positive tenant id and one canonical positive organization id.
- Every replica in the deployment uses the same scope. A process never selects scope from request
  parameters and never switches PostgreSQL scope between authenticated requests.
- PostgreSQL is the authoritative server database. Pools set `app.current_tenant_id` and
  `app.current_organization_id` before connections are established.
- App and internal RPC startup bind to the same configured pair and reject mismatched runtime scope
  before any repository or Drive dependency is selected.
- Organization-owned repository queries and mutations bind both columns. PostgreSQL RLS with
  `FORCE ROW LEVEL SECURITY` enforces the same pair as defense in depth.
- Domain `organization_id=0` denotes personal scope and is not a wildcard. The current production
  profile requires an explicit non-zero organization.
- Multiple dedicated deployments may share one PostgreSQL cluster and tables. Horizontal scaling
  occurs by adding replicas inside a fixed deployment scope or by adding dedicated deployments.
- Shared request-scoped tenant/organization pooling is unsupported and outside the current product
  scope. Enabling it would require a new reviewed requirement, ADR, migration, and release evidence.

## Alternatives

- **Request-scoped `SET LOCAL`:** rejected for this release because ordinary repositories do not
  uniformly own transaction-scoped checkout and cancellation/rollback contamination evidence is
  absent.
- **Schema or database per tenant:** rejected as the default because it multiplies migrations,
  connection pools, backup units, and operational cost without removing application auth guards.
- **Application predicates only:** rejected because a missing predicate could expose data.
- **Server SQLite:** rejected because it does not satisfy the production concurrency and RLS model.

## Consequences

- Isolation is fail-closed at IAM context, runtime guard, repository predicate, and RLS layers.
- Deployments cannot aggregate unrelated customer traffic into one request-processing pool.
- Operators must provision and route tenant/organization deployments explicitly. The per-process
  connection budget is split across one typed pool and the temporary compatibility pool; cluster
  capacity still must account for every replica plus operational reserve.
- The organization isolation migration and this security/topology decision require human review
  before merge or release.

## Verification

- Static repository inventory rejects ordinary organization-owned SQL without both predicates.
- HTTP tests reject tenant and organization mismatch, including personal-scope/non-zero mismatch.
- Real PostgreSQL tests use a non-owner role to prove same-tenant cross-organization reads and writes
  are denied.
- Deployment checks require both ids, PostgreSQL, bounded connection settings, and consistent API/
  worker scope.
- Load, soak, failover, cancellation, and OOM tests record release-candidate capacity evidence.

## Supersedes / Superseded By

When approved, this decision supersedes
[ADR-20260624-phase2-postgres-rls-multi-tenant.md](ADR-20260624-phase2-postgres-rls-multi-tenant.md).
It does not become release authority until human review accepts the proposal.
