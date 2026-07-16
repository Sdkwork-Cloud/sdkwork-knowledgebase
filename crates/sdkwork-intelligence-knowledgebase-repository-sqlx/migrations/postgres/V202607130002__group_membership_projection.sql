-- Durable reservation boundary for IM membership snapshots projected to direct Drive ACLs.
-- The source event itself is written to kb_group_knowledge_space_event_inbox only after this
-- projection commits, so a failed or interrupted projection remains retryable without replaying
-- an already-applied event.

CREATE TABLE IF NOT EXISTS kb_group_knowledge_space_membership_projection (
    id BIGINT NOT NULL,
    uuid VARCHAR(64) NOT NULL,
    tenant_id BIGINT NOT NULL,
    organization_id BIGINT NOT NULL,
    binding_id BIGINT NOT NULL,
    source_event_id VARCHAR(512) NOT NULL,
    payload_sha256_hex VARCHAR(64) NOT NULL,
    target_membership_epoch BIGINT NOT NULL,
    projection_state VARCHAR(32) NOT NULL,
    projection_lease_token VARCHAR(64),
    projection_lease_until TIMESTAMP,
    last_error_code VARCHAR(64),
    created_at TIMESTAMP NOT NULL,
    updated_at TIMESTAMP NOT NULL,
    version BIGINT NOT NULL DEFAULT 0,
    PRIMARY KEY (id),
    CONSTRAINT ck_kb_group_knowledge_space_membership_projection_state
        CHECK (projection_state IN ('pending', 'failed', 'completed')),
    CONSTRAINT ck_kb_group_knowledge_space_membership_projection_epoch
        CHECK (target_membership_epoch >= 0),
    CONSTRAINT ck_kb_group_knowledge_space_membership_projection_tenant
        CHECK (tenant_id > 0),
    CONSTRAINT ck_kb_group_knowledge_space_membership_projection_organization
        CHECK (organization_id >= 0),
    FOREIGN KEY (tenant_id, organization_id, binding_id)
        REFERENCES kb_group_knowledge_space_binding(tenant_id, organization_id, id)
);

CREATE UNIQUE INDEX IF NOT EXISTS uk_kb_group_knowledge_space_membership_projection_uuid
    ON kb_group_knowledge_space_membership_projection (tenant_id, uuid);

CREATE UNIQUE INDEX IF NOT EXISTS uk_kb_group_knowledge_space_membership_projection_event
    ON kb_group_knowledge_space_membership_projection (tenant_id, organization_id, source_event_id);

CREATE UNIQUE INDEX IF NOT EXISTS uk_kb_group_knowledge_space_membership_projection_unsettled
    ON kb_group_knowledge_space_membership_projection (tenant_id, organization_id, binding_id)
    WHERE projection_state IN ('pending', 'failed');

CREATE INDEX IF NOT EXISTS idx_kb_group_knowledge_space_membership_projection_lease
    ON kb_group_knowledge_space_membership_projection
       (tenant_id, organization_id, binding_id, projection_state, projection_lease_until);

DO $$
BEGIN
    ALTER TABLE kb_group_knowledge_space_membership_projection
        ALTER COLUMN organization_id DROP DEFAULT;
    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint
        WHERE conrelid = 'kb_group_knowledge_space_membership_projection'::regclass
          AND conname = 'ck_kb_group_knowledge_space_membership_projection_tenant'
    ) THEN
        ALTER TABLE kb_group_knowledge_space_membership_projection
            ADD CONSTRAINT ck_kb_group_knowledge_space_membership_projection_tenant
            CHECK (tenant_id > 0);
    END IF;
    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint
        WHERE conrelid = 'kb_group_knowledge_space_membership_projection'::regclass
          AND conname = 'ck_kb_group_knowledge_space_membership_projection_organization'
    ) THEN
        ALTER TABLE kb_group_knowledge_space_membership_projection
            ADD CONSTRAINT ck_kb_group_knowledge_space_membership_projection_organization
            CHECK (organization_id >= 0);
    END IF;
END $$;

ALTER TABLE kb_group_knowledge_space_membership_projection ENABLE ROW LEVEL SECURITY;
ALTER TABLE kb_group_knowledge_space_membership_projection FORCE ROW LEVEL SECURITY;

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1
        FROM pg_policies
        WHERE schemaname = current_schema()
          AND tablename = 'kb_group_knowledge_space_membership_projection'
          AND policyname = 'tenant_isolation'
    ) THEN
        CREATE POLICY tenant_isolation
            ON kb_group_knowledge_space_membership_projection
            AS PERMISSIVE
            FOR ALL
            TO PUBLIC
            USING (tenant_id = current_setting('app.current_tenant_id', true)::bigint)
            WITH CHECK (tenant_id = current_setting('app.current_tenant_id', true)::bigint);
    END IF;
END $$;
