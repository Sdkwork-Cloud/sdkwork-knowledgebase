# Provider Binding Prelaunch Readiness Runbook

Status: active prelaunch procedure  
Owner: SDKWork Knowledgebase operators  
Decision: `ADR-20260720`

## Scope

Find active `external` Knowledgebase spaces that do not have an active Provider Binding before a
tenant is admitted to a pilot or release candidate. The command is informational and read-only. It
does not install or migrate schema, inspect Provider credentials, read remote-resource identifiers,
read `kb_source`, create Bindings, or choose a Provider.

## Preconditions

- Use an approved read-capable database identity for the target environment.
- Inject `SDKWORK_KNOWLEDGEBASE_DATABASE_URL` through the environment's approved configuration or
  secret mechanism. Do not place a credential-bearing URL in command arguments, shell history, or
  the generated report.
- Set `SDKWORK_KNOWLEDGEBASE_TENANT_ID` to the canonical positive decimal tenant ID. PostgreSQL
  connections bind this value to the RLS tenant session.
- Obtain the exact nonnegative organization ID from the authorized tenant inventory. Run every
  organization scope independently.
- Confirm the release candidate schema already contains `kb_space` and `kb_provider_binding`. A
  missing table fails closed; this command never repairs the database.

## Run One Page

```powershell
$env:SDKWORK_KNOWLEDGEBASE_DATABASE_URL = '<approved-database-url>'
$env:SDKWORK_KNOWLEDGEBASE_TENANT_ID = '<tenant-id>'
cargo run -p sdkwork-knowledgebase-worker --bin sdkwork-knowledgebase-provider-binding-prelaunch-report -- --organization-id <organization-id> --page-size 20
```

The maximum `page-size` is `200`; the default is `20`. To continue, pass the returned opaque token
unchanged:

```powershell
cargo run -p sdkwork-knowledgebase-worker --bin sdkwork-knowledgebase-provider-binding-prelaunch-report -- --organization-id <organization-id> --page-size 20 --cursor '<nextCursor>'
```

Never decode, edit, construct, or reuse a cursor for another tenant or organization. Scope-mismatched,
malformed, numeric, overlong, or unsupported-version cursors are rejected.

## Output Contract

The command writes one JSON document to stdout with:

- `kind = sdkwork.knowledgebase.provider-binding-prelaunch-report` and `schemaVersion = 1.0.0`.
- Decimal-string `tenantId`, `organizationId`, `spaceId`, and `nonActiveBindingCount` fields.
- `criteria` fixed to active spaces, `external` knowledge mode, and required active Binding.
- `safety` flags proving the read-only, no-inference, no-source-order, no-credential, and
  no-remote-resource posture.
- `pageInfo.mode = cursor`, the bounded page size, `hasMore`, and optional `nextCursor`.

Each item is actionable work. `nonActiveBindingCount = 0` means the space has no retained Binding;
a positive value means only non-active lifecycle records exist. Neither value authorizes automated
Provider selection.

## Reconciliation

1. Continue until `pageInfo.hasMore` is false for the exact tenant and organization scope.
2. For every item, use the protected Provider management UI or composed backend SDK to explicitly
   create or update a draft Binding, test required capabilities, and activate it.
3. Do not populate `implementationId`, remote resource, or credential reference from historic
   source order. Those values require administrator-owned Provider inventory and review.
4. Rerun the report from the first page after remediation. Release evidence requires an empty
   first page with `hasMore = false` for every in-scope organization.
5. Record the secret-free JSON report, release artifact identity, environment, operator, reviewer,
   and execution time in the approved release evidence system. Do not commit tenant runtime data to
   this repository.

An empty report closes only the Binding data-readiness check. It does not satisfy live Provider
certification, retrieval-quality, load/SLO, PostgreSQL migration/rollback, licensing, privacy,
supply-chain, operator UI acceptance, or release approval gates.

## Failure Handling

- Invalid scope or cursor: correct the operator input; do not bypass validation.
- Database connection failure: verify approved environment injection and network policy without
  printing the connection URL.
- Query failure: confirm the release candidate schema and database health. Use the database drift
  and migration runbooks; this command must remain read-only.
- Unexpected non-empty result after activation: verify that the Binding is active, has `status = 1`,
  and matches the same tenant, organization, and space. Do not edit rows manually.

## Verification

```bash
cargo test -p sdkwork-intelligence-knowledgebase-repository-sqlx --test provider_binding_readiness_store
cargo test -p sdkwork-intelligence-knowledgebase-repository-sqlx --test provider_binding_readiness_postgres_optional
cargo test -p sdkwork-knowledgebase-worker --bin sdkwork-knowledgebase-provider-binding-prelaunch-report
node ../sdkwork-specs/tools/check-pagination.mjs --workspace .
```

The PostgreSQL test is optional locally and executes only bounded `SELECT` queries when an explicit
PostgreSQL database URL and tenant scope are configured.
