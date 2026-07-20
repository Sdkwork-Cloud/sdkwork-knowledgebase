# Migration Rollback Runbook

Status: active  
Owner: SDKWork Knowledgebase operators

## Scope

Respond to failed PostgreSQL schema migrations and recoverable Provider-to-Provider migration
operations for the Knowledgebase module.

## Provider Migration Operations

1. Confirm the target Binding references a pre-provisioned remote resource and has a successful
   health/search capability test. The Worker does not copy remote data.
2. Create the migration with a unique idempotency key, exact source/target Binding versions, and an
   observation window between 60 and 604800 seconds through
   `POST /backend/v3/api/knowledge/spaces/{spaceId}/provider_migrations`.
3. Monitor the resource returned by the list/retrieve operations. Normal progression is
   `dry_run -> preparing -> validating -> cutover -> observing -> completed`.
4. Treat `failed` as a durable terminal state requiring operator review. Do not mutate checkpoint,
   claim owner/token, lease, or Binding rows manually.
5. During `observing`, or after a recoverable pre-cutover failure, request rollback with the current
   operation version through the `/rollback` command. The Worker transitions `rolling_back ->
   rolled_back` and atomically restores the retained predecessor when cutover occurred.
6. Correlate `knowledge.provider_migration.transition` audit events and Worker structured counts by
   operation ID. Audit data intentionally excludes checkpoint, remote resource, and credential data.

Worker controls:

- `SDKWORK_KNOWLEDGEBASE_WORKER_PROVIDER_MIGRATION_BATCH_SIZE`: `1..=200`, default `25`.
- `SDKWORK_KNOWLEDGEBASE_WORKER_PROVIDER_MIGRATION_LEASE_SECONDS`: `5..=3600`, default `120`.
- `SDKWORK_KNOWLEDGEBASE_WORKER_ID`: stable replica identity; stale tokens are fenced after lease
  recovery.

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
- Provider migration repository and Hosted runtime rollback tests pass.
- The active Binding matches the target after cutover or the predecessor after rollback.
- No predecessor Binding, remote resource, credential reference, or audit evidence was deleted.
