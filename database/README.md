# KNOWLEDGEBASE Database Module

Canonical lifecycle assets for `sdkwork-knowledgebase` per `DATABASE_FRAMEWORK_SPEC.md`.

- moduleId: `knowledgebase`
- serviceCode: `KNOWLEDGEBASE`
- tablePrefixes: `kb_` for Knowledgebase-owned tables, `web_` for embedded SDKWork Web Framework audit tables.
- baselineAnchorTable: `kb_space`

## Initialization State

This module is in **initialization state** for greenfield deployments:

1. **Baseline** - `database/ddl/baseline/{engine}/0001_knowledgebase_baseline.sql` contains the full engine-specific DDL snapshot.
2. **Migrations** - `database/migrations/{engine}/` is reserved for post-GA incremental schema changes only. It is intentionally empty at initialization.
3. **Drift** - run `pnpm db:drift:check` before release. Business tables are not ignored by drift policy.

SQLite and PostgreSQL baselines are maintained separately because SQLite uses TEXT/REAL storage, FTS5 virtual tables, and folded pre-GA columns instead of PostgreSQL extensions, `JSONB`, `tsvector`, RLS, or `ALTER TABLE ... ADD COLUMN IF NOT EXISTS`.

## Commands

```bash
pnpm run db:validate
pnpm run db:materialize:contract
pnpm run db:plan
pnpm run db:init
pnpm run db:migrate
pnpm run db:seed
pnpm run db:status
pnpm run db:drift:check
```
