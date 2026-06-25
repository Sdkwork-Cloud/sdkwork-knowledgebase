-- Durable append-oriented audit events for security-relevant knowledge mutations.

CREATE TABLE IF NOT EXISTS kb_audit_event (
    id INTEGER PRIMARY KEY,
    uuid TEXT NOT NULL,
    tenant_id INTEGER NOT NULL,
    event_type TEXT NOT NULL,
    actor_type TEXT NOT NULL,
    actor_id TEXT NOT NULL,
    resource_type TEXT NOT NULL,
    resource_id INTEGER,
    result TEXT NOT NULL,
    request_id TEXT,
    trace_id TEXT,
    ip_hash TEXT,
    user_agent_hash TEXT,
    payload TEXT,
    created_at TEXT NOT NULL,
    version INTEGER NOT NULL DEFAULT 0
);

CREATE UNIQUE INDEX IF NOT EXISTS uk_kb_audit_event_uuid
    ON kb_audit_event (tenant_id, uuid);

CREATE INDEX IF NOT EXISTS idx_kb_audit_event_tenant_created
    ON kb_audit_event (tenant_id, created_at DESC);

CREATE INDEX IF NOT EXISTS idx_kb_audit_event_resource
    ON kb_audit_event (tenant_id, resource_type, resource_id);

CREATE INDEX IF NOT EXISTS idx_kb_audit_event_event_type
    ON kb_audit_event (tenant_id, event_type, created_at DESC);
