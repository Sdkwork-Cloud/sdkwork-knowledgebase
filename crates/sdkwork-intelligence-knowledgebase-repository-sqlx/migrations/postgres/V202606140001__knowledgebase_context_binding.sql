-- Knowledge space context binding: maps spaces to external contexts
-- (chat groups, organizations, circles, channels, etc.)
-- Members are NOT stored here. All permission management is delegated to
-- sdkwork-drive's dr_drive_node_permission table.

CREATE TABLE IF NOT EXISTS kb_space_context_binding (
    id BIGINT NOT NULL,
    tenant_id VARCHAR(64) NOT NULL,
    space_id BIGINT NOT NULL,
    context_type VARCHAR(32) NOT NULL,
    context_id VARCHAR(128) NOT NULL,
    context_name VARCHAR(256),
    access_level VARCHAR(32) NOT NULL DEFAULT 'reader',
    status INTEGER NOT NULL DEFAULT 1,
    created_by VARCHAR(128) NOT NULL,
    created_at TIMESTAMP NOT NULL,
    updated_at TIMESTAMP NOT NULL,
    version INTEGER NOT NULL DEFAULT 0,
    PRIMARY KEY (id),
    FOREIGN KEY (space_id) REFERENCES kb_space(id)
);

-- Prevent duplicate bindings for the same space-context pair
CREATE UNIQUE INDEX IF NOT EXISTS uk_kb_space_context
    ON kb_space_context_binding (tenant_id, space_id, context_type, context_id)
    WHERE status = 1;

-- Fast lookup: what spaces are bound to a given context?
CREATE INDEX IF NOT EXISTS idx_kb_space_context_lookup
    ON kb_space_context_binding (tenant_id, context_type, context_id, status);

-- Fast lookup: what contexts are bound to a given space?
CREATE INDEX IF NOT EXISTS idx_kb_space_context_space
    ON kb_space_context_binding (tenant_id, space_id, status);
