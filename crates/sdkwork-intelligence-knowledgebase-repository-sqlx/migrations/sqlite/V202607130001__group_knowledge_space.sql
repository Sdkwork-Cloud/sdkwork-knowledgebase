-- Pre-launch compatibility schema for the authoritative IM conversation -> KB space aggregate.
-- Production greenfield deployments receive this definition from database/ddl/baseline/sqlite.

CREATE TABLE IF NOT EXISTS kb_group_knowledge_space_binding (
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

CREATE UNIQUE INDEX IF NOT EXISTS uk_kb_group_knowledge_space_binding_uuid
    ON kb_group_knowledge_space_binding (tenant_id, uuid);
CREATE UNIQUE INDEX IF NOT EXISTS uk_kb_group_knowledge_space_binding_conversation
    ON kb_group_knowledge_space_binding (tenant_id, organization_id, conversation_id);
CREATE UNIQUE INDEX IF NOT EXISTS uk_kb_group_knowledge_space_binding_space
    ON kb_group_knowledge_space_binding (space_id)
    WHERE space_id IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_kb_group_knowledge_space_binding_state
    ON kb_group_knowledge_space_binding (tenant_id, organization_id, lifecycle_state, updated_at, id);

CREATE TABLE IF NOT EXISTS kb_group_knowledge_space_member (
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

CREATE UNIQUE INDEX IF NOT EXISTS uk_kb_group_knowledge_space_member_uuid
    ON kb_group_knowledge_space_member (tenant_id, uuid);
CREATE UNIQUE INDEX IF NOT EXISTS uk_kb_group_knowledge_space_member_active
    ON kb_group_knowledge_space_member (tenant_id, organization_id, binding_id, actor_id)
    WHERE status = 1;
CREATE INDEX IF NOT EXISTS idx_kb_group_knowledge_space_member_access
    ON kb_group_knowledge_space_member (tenant_id, organization_id, binding_id, actor_id, status);

CREATE TABLE IF NOT EXISTS kb_group_knowledge_space_event_inbox (
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

CREATE UNIQUE INDEX IF NOT EXISTS uk_kb_group_knowledge_space_event_inbox_uuid
    ON kb_group_knowledge_space_event_inbox (tenant_id, uuid);
CREATE UNIQUE INDEX IF NOT EXISTS uk_kb_group_knowledge_space_event_inbox_source
    ON kb_group_knowledge_space_event_inbox (tenant_id, organization_id, source_event_id);

-- SQLite cannot add CHECK constraints to an existing table. These idempotent guards preserve
-- the same active-ACL and role-to-access invariants for pre-launch compatibility schemas.
CREATE TRIGGER IF NOT EXISTS trg_kb_group_space_active_acl_insert
BEFORE INSERT ON kb_group_knowledge_space_binding
WHEN NEW.lifecycle_state = 'active' AND NEW.acl_projection_state <> 'active'
BEGIN
    SELECT RAISE(ABORT, 'active group knowledge space requires active ACL projection');
END;

CREATE TRIGGER IF NOT EXISTS trg_kb_group_space_active_acl_update
BEFORE UPDATE OF lifecycle_state, acl_projection_state ON kb_group_knowledge_space_binding
WHEN NEW.lifecycle_state = 'active' AND NEW.acl_projection_state <> 'active'
BEGIN
    SELECT RAISE(ABORT, 'active group knowledge space requires active ACL projection');
END;

CREATE TRIGGER IF NOT EXISTS trg_kb_group_space_member_role_access_insert
BEFORE INSERT ON kb_group_knowledge_space_member
WHEN COALESCE(NEW.access_level, '') <> CASE NEW.member_role
    WHEN 'owner' THEN 'owner'
    WHEN 'admin' THEN 'writer'
    WHEN 'member' THEN 'reader'
    WHEN 'guest' THEN ''
END
BEGIN
    SELECT RAISE(ABORT, 'group member role and access level must match');
END;

CREATE TRIGGER IF NOT EXISTS trg_kb_group_space_member_role_access_update
BEFORE UPDATE OF member_role, access_level ON kb_group_knowledge_space_member
WHEN COALESCE(NEW.access_level, '') <> CASE NEW.member_role
    WHEN 'owner' THEN 'owner'
    WHEN 'admin' THEN 'writer'
    WHEN 'member' THEN 'reader'
    WHEN 'guest' THEN ''
END
BEGIN
    SELECT RAISE(ABORT, 'group member role and access level must match');
END;

-- Existing SQLite tables cannot receive a new CHECK constraint. Preserve the
-- nonnegative token-derived scope boundary for both writes and updates.
CREATE TRIGGER IF NOT EXISTS trg_kb_group_space_binding_organization_insert
BEFORE INSERT ON kb_group_knowledge_space_binding
WHEN NEW.organization_id < 0
BEGIN
    SELECT RAISE(ABORT, 'group knowledge space organization_id must not be negative');
END;

CREATE TRIGGER IF NOT EXISTS trg_kb_group_space_binding_organization_update
BEFORE UPDATE OF organization_id ON kb_group_knowledge_space_binding
WHEN NEW.organization_id < 0
BEGIN
    SELECT RAISE(ABORT, 'group knowledge space organization_id must not be negative');
END;

CREATE TRIGGER IF NOT EXISTS trg_kb_group_space_member_organization_insert
BEFORE INSERT ON kb_group_knowledge_space_member
WHEN NEW.organization_id < 0
BEGIN
    SELECT RAISE(ABORT, 'group knowledge space organization_id must not be negative');
END;

CREATE TRIGGER IF NOT EXISTS trg_kb_group_space_member_organization_update
BEFORE UPDATE OF organization_id ON kb_group_knowledge_space_member
WHEN NEW.organization_id < 0
BEGIN
    SELECT RAISE(ABORT, 'group knowledge space organization_id must not be negative');
END;

CREATE TRIGGER IF NOT EXISTS trg_kb_group_space_event_inbox_organization_insert
BEFORE INSERT ON kb_group_knowledge_space_event_inbox
WHEN NEW.organization_id < 0
BEGIN
    SELECT RAISE(ABORT, 'group knowledge space organization_id must not be negative');
END;

CREATE TRIGGER IF NOT EXISTS trg_kb_group_space_event_inbox_organization_update
BEFORE UPDATE OF organization_id ON kb_group_knowledge_space_event_inbox
WHEN NEW.organization_id < 0
BEGIN
    SELECT RAISE(ABORT, 'group knowledge space organization_id must not be negative');
END;
