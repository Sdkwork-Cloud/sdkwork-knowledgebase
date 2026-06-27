-- source: crates/sdkwork-intelligence-knowledgebase-repository-sqlx/migrations/sqlite/V202606220003__knowledgebase_outbox_claim.sql
-- Outbox worker claim timestamp for stale-claim release.

ALTER TABLE kb_outbox_event ADD COLUMN claimed_at TEXT;
