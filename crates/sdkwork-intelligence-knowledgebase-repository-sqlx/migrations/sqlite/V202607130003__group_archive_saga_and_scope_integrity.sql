-- SQLite greenfield schemas are defined by V202607130001/V202607130002. Existing pre-launch
-- tables receive additive archive-saga fields here. Trigger programs are deliberately isolated
-- in V202607130004 so this migration can be executed statement-by-statement and safely replayed
-- after an interrupted SQLite bootstrap.

ALTER TABLE kb_group_knowledge_space_binding
    ADD COLUMN upstream_link_generation INTEGER NOT NULL DEFAULT 0;
ALTER TABLE kb_group_knowledge_space_binding
    ADD COLUMN archive_source_event_id TEXT;
ALTER TABLE kb_group_knowledge_space_binding
    ADD COLUMN archive_payload_sha256_hex TEXT;
ALTER TABLE kb_group_knowledge_space_binding
    ADD COLUMN archive_lease_token TEXT;
ALTER TABLE kb_group_knowledge_space_binding
    ADD COLUMN archive_lease_until TEXT;
ALTER TABLE kb_group_knowledge_space_binding
    ADD COLUMN archive_acl_cursor TEXT;
ALTER TABLE kb_group_knowledge_space_binding
    ADD COLUMN archive_acl_pages_processed INTEGER NOT NULL DEFAULT 0;
ALTER TABLE kb_group_knowledge_space_binding
    ADD COLUMN archive_acl_cleanup_completed_at TEXT;

CREATE UNIQUE INDEX IF NOT EXISTS uk_kb_group_knowledge_space_binding_idempotency
    ON kb_group_knowledge_space_binding
       (tenant_id, organization_id, provisioning_idempotency_key_sha256_hex);
