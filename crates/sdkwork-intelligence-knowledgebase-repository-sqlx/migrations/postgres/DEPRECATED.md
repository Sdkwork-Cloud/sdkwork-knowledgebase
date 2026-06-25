# Legacy embedded SQL (read-only mirror)

These SQL files are **not** the operational migration authority.

| Purpose | Canonical location |
|---------|-------------------|
| Production PostgreSQL migrations | `database/migrations/postgres/` |
| Contract-first schema | `database/contract/schema.yaml` |
| SQLite bootstrap embedded in Rust | This directory (mirror only) |

Rules:

- **Do not add new schema files here.** Add `database/migrations/{engine}/*.up.sql` and run `pnpm db:materialize:contract`.
- `src/migrations.rs` includes these files for SQLite dev bootstrap and migration manifest tests only.
- PostgreSQL runtime bootstrap uses `sdkwork-knowledgebase-database-host` and application-root `database/`.

See `database/README.md` and `DATABASE_FRAMEWORK_SPEC.md`.
