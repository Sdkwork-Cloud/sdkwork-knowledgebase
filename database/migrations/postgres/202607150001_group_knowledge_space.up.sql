-- Backfilled expand migration for installations whose baseline predates the IM group-space
-- aggregate. The lifecycle skips mutable baselines once kb_space exists, so these tables must
-- also be created by an ordered runtime migration before later corrections alter them.

CREATE TABLE IF NOT EXISTS kb_group_knowledge_space_binding (
    id BIGINT NOT NULL,
    uuid VARCHAR(64) NOT NULL,
    tenant_id BIGINT NOT NULL,
    organization_id BIGINT NOT NULL,
    conversation_id VARCHAR(256) NOT NULL,
    space_id BIGINT,
    space_uuid VARCHAR(64),
    group_name VARCHAR(256) NOT NULL,
    lifecycle_state VARCHAR(32) NOT NULL,
    acl_projection_state VARCHAR(32) NOT NULL DEFAULT 'pending',
    provisioning_idempotency_key_sha256_hex VARCHAR(64) NOT NULL,
    provisioning_lease_token VARCHAR(64),
    provisioning_lease_until TIMESTAMP,
    membership_epoch BIGINT NOT NULL DEFAULT 0,
    upstream_link_generation BIGINT NOT NULL DEFAULT 0,
    archive_source_event_id VARCHAR(512),
    archive_payload_sha256_hex VARCHAR(64),
    archive_lease_token VARCHAR(64),
    archive_lease_until TIMESTAMP,
    archive_acl_cursor VARCHAR(2048),
    archive_acl_pages_processed BIGINT NOT NULL DEFAULT 0,
    archive_acl_cleanup_completed_at TIMESTAMP,
    last_source_event_id VARCHAR(512),
    last_error_code VARCHAR(64),
    last_error_at TIMESTAMP,
    archived_at TIMESTAMP,
    archived_by VARCHAR(256),
    deleted_at TIMESTAMP,
    deleted_by VARCHAR(256),
    created_by VARCHAR(256) NOT NULL,
    updated_by VARCHAR(256) NOT NULL,
    created_at TIMESTAMP NOT NULL,
    updated_at TIMESTAMP NOT NULL,
    version BIGINT NOT NULL DEFAULT 0,
    PRIMARY KEY (id),
    CONSTRAINT uk_kb_group_knowledge_space_binding_scope_id
        UNIQUE (tenant_id, organization_id, id),
    CONSTRAINT ck_kb_group_knowledge_space_lifecycle
        CHECK (lifecycle_state IN ('provisioning', 'active', 'failed', 'archiving', 'archived', 'deleted')),
    CONSTRAINT ck_kb_group_knowledge_space_acl_projection
        CHECK (acl_projection_state IN ('pending', 'active', 'failed')),
    CONSTRAINT ck_kb_group_knowledge_space_active_acl_projection
        CHECK (lifecycle_state <> 'active' OR acl_projection_state = 'active'),
    CONSTRAINT ck_kb_group_knowledge_space_membership_epoch
        CHECK (membership_epoch >= 0),
    CONSTRAINT ck_kb_group_knowledge_space_binding_tenant
        CHECK (tenant_id > 0),
    CONSTRAINT ck_kb_group_knowledge_space_binding_organization
        CHECK (organization_id >= 0),
    FOREIGN KEY (space_id) REFERENCES kb_space(id)
);

CREATE UNIQUE INDEX IF NOT EXISTS uk_kb_group_knowledge_space_binding_uuid
    ON kb_group_knowledge_space_binding (tenant_id, uuid);
CREATE UNIQUE INDEX IF NOT EXISTS uk_kb_group_knowledge_space_binding_conversation
    ON kb_group_knowledge_space_binding (tenant_id, organization_id, conversation_id);
CREATE UNIQUE INDEX IF NOT EXISTS uk_kb_group_knowledge_space_binding_idempotency
    ON kb_group_knowledge_space_binding
       (tenant_id, organization_id, provisioning_idempotency_key_sha256_hex);
CREATE UNIQUE INDEX IF NOT EXISTS uk_kb_group_knowledge_space_binding_space
    ON kb_group_knowledge_space_binding (space_id)
    WHERE space_id IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_kb_group_knowledge_space_binding_state
    ON kb_group_knowledge_space_binding
       (tenant_id, organization_id, lifecycle_state, updated_at, id);

CREATE TABLE IF NOT EXISTS kb_group_knowledge_space_member (
    id BIGINT NOT NULL,
    uuid VARCHAR(64) NOT NULL,
    tenant_id BIGINT NOT NULL,
    organization_id BIGINT NOT NULL,
    binding_id BIGINT NOT NULL,
    principal_kind VARCHAR(32) NOT NULL,
    actor_id VARCHAR(256) NOT NULL,
    member_role VARCHAR(32) NOT NULL,
    access_level VARCHAR(32),
    membership_epoch BIGINT NOT NULL,
    status INTEGER NOT NULL DEFAULT 1,
    created_at TIMESTAMP NOT NULL,
    updated_at TIMESTAMP NOT NULL,
    version BIGINT NOT NULL DEFAULT 0,
    PRIMARY KEY (id),
    CONSTRAINT ck_kb_group_knowledge_space_member_role
        CHECK (member_role IN ('owner', 'admin', 'member', 'guest')),
    CONSTRAINT ck_kb_group_knowledge_space_member_principal
        CHECK (principal_kind = 'user'),
    CONSTRAINT ck_kb_group_knowledge_space_member_access
        CHECK (access_level IS NULL OR access_level IN ('reader', 'writer', 'owner')),
    CONSTRAINT ck_kb_group_knowledge_space_member_role_access
        CHECK (
            COALESCE(access_level, '') = CASE member_role
                WHEN 'owner' THEN 'owner'
                WHEN 'admin' THEN 'writer'
                WHEN 'member' THEN 'reader'
                WHEN 'guest' THEN ''
            END
        ),
    CONSTRAINT ck_kb_group_knowledge_space_member_epoch
        CHECK (membership_epoch >= 0),
    CONSTRAINT ck_kb_group_knowledge_space_member_tenant
        CHECK (tenant_id > 0),
    CONSTRAINT ck_kb_group_knowledge_space_member_organization
        CHECK (organization_id >= 0),
    FOREIGN KEY (tenant_id, organization_id, binding_id)
        REFERENCES kb_group_knowledge_space_binding(tenant_id, organization_id, id)
);

CREATE UNIQUE INDEX IF NOT EXISTS uk_kb_group_knowledge_space_member_uuid
    ON kb_group_knowledge_space_member (tenant_id, uuid);
CREATE UNIQUE INDEX IF NOT EXISTS uk_kb_group_knowledge_space_member_active
    ON kb_group_knowledge_space_member (tenant_id, organization_id, binding_id, actor_id)
    WHERE status = 1;
CREATE INDEX IF NOT EXISTS idx_kb_group_knowledge_space_member_access
    ON kb_group_knowledge_space_member
       (tenant_id, organization_id, binding_id, actor_id, status);

CREATE TABLE IF NOT EXISTS kb_group_knowledge_space_event_inbox (
    id BIGINT NOT NULL,
    uuid VARCHAR(64) NOT NULL,
    tenant_id BIGINT NOT NULL,
    organization_id BIGINT NOT NULL,
    source_event_id VARCHAR(512) NOT NULL,
    event_type VARCHAR(64) NOT NULL,
    binding_id BIGINT,
    payload_sha256_hex VARCHAR(64) NOT NULL,
    applied_at TIMESTAMP NOT NULL,
    PRIMARY KEY (id),
    CONSTRAINT ck_kb_group_knowledge_space_event_inbox_tenant
        CHECK (tenant_id > 0),
    CONSTRAINT ck_kb_group_knowledge_space_event_inbox_organization
        CHECK (organization_id >= 0),
    FOREIGN KEY (tenant_id, organization_id, binding_id)
        REFERENCES kb_group_knowledge_space_binding(tenant_id, organization_id, id)
);

CREATE UNIQUE INDEX IF NOT EXISTS uk_kb_group_knowledge_space_event_inbox_uuid
    ON kb_group_knowledge_space_event_inbox (tenant_id, uuid);
CREATE UNIQUE INDEX IF NOT EXISTS uk_kb_group_knowledge_space_event_inbox_source
    ON kb_group_knowledge_space_event_inbox (tenant_id, organization_id, source_event_id);

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
    ON kb_group_knowledge_space_membership_projection
       (tenant_id, organization_id, source_event_id);
CREATE UNIQUE INDEX IF NOT EXISTS uk_kb_group_knowledge_space_membership_projection_unsettled
    ON kb_group_knowledge_space_membership_projection
       (tenant_id, organization_id, binding_id)
    WHERE projection_state IN ('pending', 'failed');
CREATE INDEX IF NOT EXISTS idx_kb_group_knowledge_space_membership_projection_lease
    ON kb_group_knowledge_space_membership_projection
       (tenant_id, organization_id, binding_id, projection_state, projection_lease_until);

DO $$
DECLARE
    table_name text;
BEGIN
    FOREACH table_name IN ARRAY ARRAY[
        'kb_group_knowledge_space_binding',
        'kb_group_knowledge_space_member',
        'kb_group_knowledge_space_event_inbox',
        'kb_group_knowledge_space_membership_projection'
    ]
    LOOP
        EXECUTE format('ALTER TABLE %I ENABLE ROW LEVEL SECURITY', table_name);
        EXECUTE format('ALTER TABLE %I FORCE ROW LEVEL SECURITY', table_name);
        IF NOT EXISTS (
            SELECT 1
            FROM pg_policies
            WHERE schemaname = current_schema()
              AND tablename = table_name
              AND policyname = 'tenant_isolation'
        ) THEN
            EXECUTE format(
                'CREATE POLICY tenant_isolation ON %I AS PERMISSIVE FOR ALL TO PUBLIC USING (tenant_id = current_setting(''app.current_tenant_id'', true)::bigint) WITH CHECK (tenant_id = current_setting(''app.current_tenant_id'', true)::bigint)',
                table_name
            );
        END IF;
    END LOOP;
END $$;
