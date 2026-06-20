# KNOWLEDGEBASE Database Module

Canonical lifecycle assets for `sdkwork-knowledgebase` per `DATABASE_FRAMEWORK_SPEC.md`.

- moduleId: `knowledgebase`
- serviceCode: `KNOWLEDGEBASE`
- tablePrefix: `kb_`

## Commands

```bash
pnpm run db:validate
pnpm run db:plan
pnpm run db:init
pnpm run db:migrate
pnpm run db:seed
pnpm run db:status
pnpm run db:drift:check
```

## Migration status

Legacy SQL was consolidated into `ddl/baseline/postgres/0001_*_legacy_baseline.sql` for bootstrap review.
Author contract-first tables in `contract/schema.yaml`, then split baseline into versioned `migrations/` pairs.

Imported legacy sources:
- `crates/sdkwork-intelligence-knowledgebase-repository-sqlx/migrations/postgres/V202606010001__knowledgebase_core.sql`
- `crates/sdkwork-intelligence-knowledgebase-repository-sqlx/migrations/postgres/V202606140001__knowledgebase_context_binding.sql`
- `crates/sdkwork-intelligence-knowledgebase-repository-sqlx/migrations/postgres/V202606170001__knowledge_access_mode.sql`
- `crates/sdkwork-intelligence-knowledgebase-repository-sqlx/migrations/postgres/V202606180001__agent_implementation.sql`
- `crates/sdkwork-intelligence-knowledgebase-repository-sqlx/migrations/postgres/V202606190001__knowledgebase_pgvector.sql`
- `crates/sdkwork-intelligence-knowledgebase-repository-sqlx/migrations/postgres/V202606200001__knowledgebase_outbox.sql`
- `crates/sdkwork-intelligence-knowledgebase-repository-sqlx/migrations/postgres/V202606210001__okf_link_and_candidate.sql`

Runtime services MUST create pools through `sdkwork-database-sqlx` and register `DefaultDatabaseModule` at bootstrap via `sdkwork-knowledgebase-database-host`.

```bash
pnpm run db:materialize:contract
```
