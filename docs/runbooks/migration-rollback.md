# Migration Rollback Runbook

Status: active  
Owner: SDKWork Knowledgebase operators

## Scope

Respond to failed PostgreSQL migrations for the Knowledgebase module.

## Preconditions

- Latest migration status from `pnpm db:status`.
- Database backup captured per `deployments/runbooks/backup-restore.md`.

## Forward-fix preferred

1. Identify failing migration under `database/migrations/postgres/`.
2. Ship a corrective forward migration rather than destructive rollback when data exists.
3. Run `pnpm db:migrate` in the target environment.
4. Validate with `pnpm db:drift:check` and `/readyz`.

## Approved down migration

When `*.down.sql` exists (for example `0005_knowledgebase_audit_event.down.sql`):

1. Stop API and worker processes.
2. Apply the down script with your approved SQL runner against the tenant database.
3. Redeploy the previous application artifact.
4. Run `pnpm db:status` and confirm schema version alignment.

## Rollback forbidden

Do not manually drop `kb_*` core tables in production without platform approval and a verified backup restore plan.

## Verification

- `pnpm db:validate` passes locally against the target contract.
- Application `/readyz` returns success.
- Smoke retrieval and document save succeed.
