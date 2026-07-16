-- Durable reservation boundary for IM membership snapshots projected to direct Drive ACLs.
-- The source event itself is written to kb_group_knowledge_space_event_inbox only after this
-- projection commits, so a failed or interrupted projection remains retryable without replaying
-- an already-applied event.

CREATE TABLE IF NOT EXISTS kb_group_knowledge_space_membership_projection (
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

CREATE TRIGGER IF NOT EXISTS trg_kb_group_space_membership_projection_organization_insert
BEFORE INSERT ON kb_group_knowledge_space_membership_projection
WHEN NEW.organization_id < 0
BEGIN
    SELECT RAISE(ABORT, 'group knowledge space organization_id must not be negative');
END;

CREATE TRIGGER IF NOT EXISTS trg_kb_group_space_membership_projection_organization_update
BEFORE UPDATE OF organization_id ON kb_group_knowledge_space_membership_projection
WHEN NEW.organization_id < 0
BEGIN
    SELECT RAISE(ABORT, 'group knowledge space organization_id must not be negative');
END;
