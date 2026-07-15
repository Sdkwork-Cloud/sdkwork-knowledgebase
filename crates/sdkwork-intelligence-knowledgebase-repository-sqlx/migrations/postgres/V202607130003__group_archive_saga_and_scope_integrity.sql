-- Durable archive-saga state, IM wire-length alignment, and tenant/organization child-scope
-- integrity for existing pre-launch Knowledgebase installations.

ALTER TABLE kb_group_knowledge_space_binding
    ALTER COLUMN conversation_id TYPE VARCHAR(256),
    ALTER COLUMN last_source_event_id TYPE VARCHAR(512),
    ALTER COLUMN archived_by TYPE VARCHAR(256),
    ALTER COLUMN deleted_by TYPE VARCHAR(256),
    ALTER COLUMN created_by TYPE VARCHAR(256),
    ALTER COLUMN updated_by TYPE VARCHAR(256),
    ADD COLUMN IF NOT EXISTS upstream_link_generation BIGINT NOT NULL DEFAULT 0,
    ADD COLUMN IF NOT EXISTS archive_source_event_id VARCHAR(512),
    ADD COLUMN IF NOT EXISTS archive_payload_sha256_hex VARCHAR(64),
    ADD COLUMN IF NOT EXISTS archive_lease_token VARCHAR(64),
    ADD COLUMN IF NOT EXISTS archive_lease_until TIMESTAMP,
    ADD COLUMN IF NOT EXISTS archive_acl_cursor VARCHAR(2048),
    ADD COLUMN IF NOT EXISTS archive_acl_pages_processed BIGINT NOT NULL DEFAULT 0,
    ADD COLUMN IF NOT EXISTS archive_acl_cleanup_completed_at TIMESTAMP;

ALTER TABLE kb_group_knowledge_space_member
    ALTER COLUMN actor_id TYPE VARCHAR(256);

ALTER TABLE kb_group_knowledge_space_event_inbox
    ALTER COLUMN source_event_id TYPE VARCHAR(512);

ALTER TABLE kb_group_knowledge_space_membership_projection
    ALTER COLUMN source_event_id TYPE VARCHAR(512);

ALTER TABLE kb_group_knowledge_space_binding
    DROP CONSTRAINT IF EXISTS ck_kb_group_knowledge_space_lifecycle;
ALTER TABLE kb_group_knowledge_space_binding
    ADD CONSTRAINT ck_kb_group_knowledge_space_lifecycle
    CHECK (lifecycle_state IN ('provisioning', 'active', 'failed', 'archiving', 'archived', 'deleted'));

ALTER TABLE kb_group_knowledge_space_membership_projection
    DROP CONSTRAINT IF EXISTS ck_kb_group_knowledge_space_membership_projection_state;
ALTER TABLE kb_group_knowledge_space_membership_projection
    ADD CONSTRAINT ck_kb_group_knowledge_space_membership_projection_state
    CHECK (projection_state IN ('pending', 'failed', 'completed'));

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint
        WHERE conrelid = 'kb_group_knowledge_space_binding'::regclass
          AND conname = 'uk_kb_group_knowledge_space_binding_scope_id'
    ) THEN
        ALTER TABLE kb_group_knowledge_space_binding
            ADD CONSTRAINT uk_kb_group_knowledge_space_binding_scope_id
            UNIQUE (tenant_id, organization_id, id);
    END IF;

    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint
        WHERE conrelid = 'kb_group_knowledge_space_member'::regclass
          AND conname = 'fk_kb_group_knowledge_space_member_scope_binding'
    ) THEN
        ALTER TABLE kb_group_knowledge_space_member
            ADD CONSTRAINT fk_kb_group_knowledge_space_member_scope_binding
            FOREIGN KEY (tenant_id, organization_id, binding_id)
            REFERENCES kb_group_knowledge_space_binding(tenant_id, organization_id, id);
    END IF;

    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint
        WHERE conrelid = 'kb_group_knowledge_space_event_inbox'::regclass
          AND conname = 'fk_kb_group_knowledge_space_event_scope_binding'
    ) THEN
        ALTER TABLE kb_group_knowledge_space_event_inbox
            ADD CONSTRAINT fk_kb_group_knowledge_space_event_scope_binding
            FOREIGN KEY (tenant_id, organization_id, binding_id)
            REFERENCES kb_group_knowledge_space_binding(tenant_id, organization_id, id);
    END IF;

    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint
        WHERE conrelid = 'kb_group_knowledge_space_membership_projection'::regclass
          AND conname = 'fk_kb_group_knowledge_space_projection_scope_binding'
    ) THEN
        ALTER TABLE kb_group_knowledge_space_membership_projection
            ADD CONSTRAINT fk_kb_group_knowledge_space_projection_scope_binding
            FOREIGN KEY (tenant_id, organization_id, binding_id)
            REFERENCES kb_group_knowledge_space_binding(tenant_id, organization_id, id);
    END IF;
END $$;

CREATE UNIQUE INDEX IF NOT EXISTS uk_kb_group_knowledge_space_binding_idempotency
    ON kb_group_knowledge_space_binding
       (tenant_id, organization_id, provisioning_idempotency_key_sha256_hex);
