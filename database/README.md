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

## Migration authority

| Location | Role |
|----------|------|
| `database/migrations/{engine}/` | **Canonical** versioned migrations for production (`pnpm db:migrate`) |
| `database/contract/schema.yaml` | Contract-first schema source |
| `crates/.../migrations/` | **Legacy mirror** for SQLite bootstrap and migration manifest tests only — do not add new files |

New schema changes: edit `database/contract/schema.yaml`, add `database/migrations/{engine}/*.up.sql`, run `pnpm db:materialize:contract` and `pnpm db:drift:check`.

Runtime services create pools through `sdkwork-database-sqlx` and register `DefaultDatabaseModule` at bootstrap via `sdkwork-knowledgebase-database-host`.
