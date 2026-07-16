ALTER TABLE kb_group_knowledge_space_membership_projection
    RENAME TO kb_group_knowledge_space_membership_projection_tenant_scope_old;
ALTER TABLE kb_group_knowledge_space_event_inbox
    RENAME TO kb_group_knowledge_space_event_inbox_tenant_scope_old;
ALTER TABLE kb_group_knowledge_space_member
    RENAME TO kb_group_knowledge_space_member_tenant_scope_old;
ALTER TABLE kb_group_knowledge_space_binding
    RENAME TO kb_group_knowledge_space_binding_tenant_scope_old;

CREATE TABLE kb_group_knowledge_space_binding (
    id INTEGER NOT NULL,
    uuid TEXT NOT NULL,
    tenant_id INTEGER NOT NULL,
    organization_id INTEGER NOT NULL,
    conversation_id TEXT NOT NULL,
    space_id INTEGER,
    space_uuid TEXT,
    group_name TEXT NOT NULL,
    lifecycle_state TEXT NOT NULL,
    acl_projection_state TEXT NOT NULL DEFAULT 'pending',
    provisioning_idempotency_key_sha256_hex TEXT NOT NULL,
    provisioning_lease_token TEXT,
    provisioning_lease_until TEXT,
    membership_epoch INTEGER NOT NULL DEFAULT 0,
    upstream_link_generation INTEGER NOT NULL DEFAULT 0,
    archive_source_event_id TEXT,
    archive_payload_sha256_hex TEXT,
    archive_lease_token TEXT,
    archive_lease_until TEXT,
    archive_acl_cursor TEXT,
    archive_acl_pages_processed INTEGER NOT NULL DEFAULT 0,
    archive_acl_cleanup_completed_at TEXT,
    last_source_event_id TEXT,
    last_error_code TEXT,
    last_error_at TEXT,
    archived_at TEXT,
    archived_by TEXT,
    deleted_at TEXT,
    deleted_by TEXT,
    created_by TEXT NOT NULL,
    updated_by TEXT NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    version INTEGER NOT NULL DEFAULT 0,
    PRIMARY KEY (id),
    UNIQUE (tenant_id, organization_id, id),
    CHECK (lifecycle_state IN ('provisioning', 'active', 'failed', 'archiving', 'archived', 'deleted')),
    CHECK (acl_projection_state IN ('pending', 'active', 'failed')),
    CHECK (lifecycle_state <> 'active' OR acl_projection_state = 'active'),
    CHECK (membership_epoch >= 0),
    CHECK (tenant_id > 0),
    CHECK (organization_id >= 0),
    FOREIGN KEY (space_id) REFERENCES kb_space(id)
);

CREATE TABLE kb_group_knowledge_space_member (
    id INTEGER NOT NULL,
    uuid TEXT NOT NULL,
    tenant_id INTEGER NOT NULL,
    organization_id INTEGER NOT NULL,
    binding_id INTEGER NOT NULL,
    principal_kind TEXT NOT NULL,
    actor_id TEXT NOT NULL,
    member_role TEXT NOT NULL,
    access_level TEXT,
    membership_epoch INTEGER NOT NULL,
    status INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    version INTEGER NOT NULL DEFAULT 0,
    PRIMARY KEY (id),
    CHECK (member_role IN ('owner', 'admin', 'member', 'guest')),
    CHECK (principal_kind = 'user'),
    CHECK (access_level IS NULL OR access_level IN ('reader', 'writer', 'owner')),
    CHECK (
        COALESCE(access_level, '') = CASE member_role
            WHEN 'owner' THEN 'owner'
            WHEN 'admin' THEN 'writer'
            WHEN 'member' THEN 'reader'
            WHEN 'guest' THEN ''
        END
    ),
    CHECK (membership_epoch >= 0),
    CHECK (tenant_id > 0),
    CHECK (organization_id >= 0),
    FOREIGN KEY (tenant_id, organization_id, binding_id)
        REFERENCES kb_group_knowledge_space_binding(tenant_id, organization_id, id)
);

CREATE TABLE kb_group_knowledge_space_event_inbox (
    id INTEGER NOT NULL,
    uuid TEXT NOT NULL,
    tenant_id INTEGER NOT NULL,
    organization_id INTEGER NOT NULL,
    source_event_id TEXT NOT NULL,
    event_type TEXT NOT NULL,
    binding_id INTEGER,
    payload_sha256_hex TEXT NOT NULL,
    applied_at TEXT NOT NULL,
    PRIMARY KEY (id),
    CHECK (tenant_id > 0),
    CHECK (organization_id >= 0),
    FOREIGN KEY (tenant_id, organization_id, binding_id)
        REFERENCES kb_group_knowledge_space_binding(tenant_id, organization_id, id)
);

CREATE TABLE kb_group_knowledge_space_membership_projection (
    id INTEGER NOT NULL,
    uuid TEXT NOT NULL,
    tenant_id INTEGER NOT NULL,
    organization_id INTEGER NOT NULL,
    binding_id INTEGER NOT NULL,
    source_event_id TEXT NOT NULL,
    payload_sha256_hex TEXT NOT NULL,
    target_membership_epoch INTEGER NOT NULL,
    projection_state TEXT NOT NULL,
    projection_lease_token TEXT,
    projection_lease_until TEXT,
    last_error_code TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    version INTEGER NOT NULL DEFAULT 0,
    PRIMARY KEY (id),
    CHECK (projection_state IN ('pending', 'failed', 'completed')),
    CHECK (target_membership_epoch >= 0),
    CHECK (tenant_id > 0),
    CHECK (organization_id >= 0),
    FOREIGN KEY (tenant_id, organization_id, binding_id)
        REFERENCES kb_group_knowledge_space_binding(tenant_id, organization_id, id)
);

INSERT INTO kb_group_knowledge_space_binding (
    id, uuid, tenant_id, organization_id, conversation_id, space_id, space_uuid, group_name,
    lifecycle_state, acl_projection_state, provisioning_idempotency_key_sha256_hex,
    provisioning_lease_token, provisioning_lease_until, membership_epoch,
    upstream_link_generation, archive_source_event_id, archive_payload_sha256_hex,
    archive_lease_token, archive_lease_until, archive_acl_cursor, archive_acl_pages_processed,
    archive_acl_cleanup_completed_at, last_source_event_id, last_error_code, last_error_at,
    archived_at, archived_by, deleted_at, deleted_by, created_by, updated_by, created_at,
    updated_at, version
)
SELECT
    id, uuid, tenant_id, organization_id, conversation_id, space_id, space_uuid, group_name,
    lifecycle_state, acl_projection_state, provisioning_idempotency_key_sha256_hex,
    provisioning_lease_token, provisioning_lease_until, membership_epoch,
    upstream_link_generation, archive_source_event_id, archive_payload_sha256_hex,
    archive_lease_token, archive_lease_until, archive_acl_cursor, archive_acl_pages_processed,
    archive_acl_cleanup_completed_at, last_source_event_id, last_error_code, last_error_at,
    archived_at, archived_by, deleted_at, deleted_by, created_by, updated_by, created_at,
    updated_at, version
FROM kb_group_knowledge_space_binding_tenant_scope_old;

INSERT INTO kb_group_knowledge_space_member (
    id, uuid, tenant_id, organization_id, binding_id, principal_kind, actor_id, member_role,
    access_level, membership_epoch, status, created_at, updated_at, version
)
SELECT
    id, uuid, tenant_id, organization_id, binding_id, principal_kind, actor_id, member_role,
    access_level, membership_epoch, status, created_at, updated_at, version
FROM kb_group_knowledge_space_member_tenant_scope_old;

INSERT INTO kb_group_knowledge_space_event_inbox (
    id, uuid, tenant_id, organization_id, source_event_id, event_type, binding_id,
    payload_sha256_hex, applied_at
)
SELECT
    id, uuid, tenant_id, organization_id, source_event_id, event_type, binding_id,
    payload_sha256_hex, applied_at
FROM kb_group_knowledge_space_event_inbox_tenant_scope_old;

INSERT INTO kb_group_knowledge_space_membership_projection (
    id, uuid, tenant_id, organization_id, binding_id, source_event_id, payload_sha256_hex,
    target_membership_epoch, projection_state, projection_lease_token, projection_lease_until,
    last_error_code, created_at, updated_at, version
)
SELECT
    id, uuid, tenant_id, organization_id, binding_id, source_event_id, payload_sha256_hex,
    target_membership_epoch, projection_state, projection_lease_token, projection_lease_until,
    last_error_code, created_at, updated_at, version
FROM kb_group_knowledge_space_membership_projection_tenant_scope_old;

DROP TABLE kb_group_knowledge_space_membership_projection_tenant_scope_old;
DROP TABLE kb_group_knowledge_space_event_inbox_tenant_scope_old;
DROP TABLE kb_group_knowledge_space_member_tenant_scope_old;
DROP TABLE kb_group_knowledge_space_binding_tenant_scope_old;

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

CREATE UNIQUE INDEX IF NOT EXISTS uk_kb_group_knowledge_space_member_uuid
    ON kb_group_knowledge_space_member (tenant_id, uuid);
CREATE UNIQUE INDEX IF NOT EXISTS uk_kb_group_knowledge_space_member_active
    ON kb_group_knowledge_space_member (tenant_id, organization_id, binding_id, actor_id)
    WHERE status = 1;
CREATE INDEX IF NOT EXISTS idx_kb_group_knowledge_space_member_access
    ON kb_group_knowledge_space_member
       (tenant_id, organization_id, binding_id, actor_id, status);

CREATE UNIQUE INDEX IF NOT EXISTS uk_kb_group_knowledge_space_event_inbox_uuid
    ON kb_group_knowledge_space_event_inbox (tenant_id, uuid);
CREATE UNIQUE INDEX IF NOT EXISTS uk_kb_group_knowledge_space_event_inbox_source
    ON kb_group_knowledge_space_event_inbox (tenant_id, organization_id, source_event_id);

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
