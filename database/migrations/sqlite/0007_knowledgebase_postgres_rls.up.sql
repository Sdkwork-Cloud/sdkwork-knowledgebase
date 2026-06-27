-- SQLite has no native Row-Level Security (RLS).
-- Tenant isolation for the SQLite engine is enforced at the application layer
-- via tenant-scoped queries (see repository-sqlx ensure_tenant_scope helpers).
-- This migration is a no-op placeholder that preserves migration ordering
-- parity with database/migrations/postgres/0007_knowledgebase_postgres_rls.up.sql.
--
-- Reference: database/database.manifest.json declares engines ["postgres", "sqlite"]
-- and the SQLite engine is intended for single-tenant development/profiles only.
SELECT 1;
