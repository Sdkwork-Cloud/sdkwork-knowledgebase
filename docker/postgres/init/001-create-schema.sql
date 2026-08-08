-- sdkwork-knowledgebase standalone compose (docker-compose.yml)
-- Creates the canonical workspace PostgreSQL schema used by the container.
-- The postgres image only creates the database from POSTGRES_DB; the
-- sdkwork-database lifecycle pins search_path to the same-named schema, so
-- the schema must exist before migrations run.
--
-- The role-level search_path default makes the workspace schema visible to
-- every connection (IAM/Drive/web-framework pools do not attach an explicit
-- search_path), while the lifecycle's explicit -c search_path=... overrides
-- it on the application pool.
CREATE SCHEMA IF NOT EXISTS sdkwork_ai_prod;
GRANT ALL ON SCHEMA sdkwork_ai_prod TO sdkwork_ai_prod;
ALTER ROLE sdkwork_ai_prod SET search_path TO sdkwork_ai_prod, public;
