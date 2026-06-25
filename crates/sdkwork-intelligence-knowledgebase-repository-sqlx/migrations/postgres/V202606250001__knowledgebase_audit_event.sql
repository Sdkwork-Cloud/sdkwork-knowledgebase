-- Durable append-oriented audit events for security-relevant knowledge mutations.

CREATE TABLE IF NOT EXISTS kb_audit_event (
    id BIGINT PRIMARY KEY,
    uuid VARCHAR(64) NOT NULL,
    tenant_id BIGINT NOT NULL,
    event_type VARCHAR(128) NOT NULL,
    actor_type VARCHAR(64) NOT NULL,
    actor_id VARCHAR(128) NOT NULL,
    resource_type VARCHAR(64) NOT NULL,
    resource_id BIGINT,
    result VARCHAR(64) NOT NULL,
    request_id VARCHAR(64),
    trace_id VARCHAR(128),
    ip_hash VARCHAR(128),
    user_agent_hash VARCHAR(128),
    payload JSONB,
    created_at TIMESTAMP NOT NULL,
    version BIGINT NOT NULL DEFAULT 0
);

CREATE UNIQUE INDEX IF NOT EXISTS uk_kb_audit_event_uuid
    ON kb_audit_event (tenant_id, uuid);

CREATE INDEX IF NOT EXISTS idx_kb_audit_event_tenant_created
    ON kb_audit_event (tenant_id, created_at DESC);

CREATE INDEX IF NOT EXISTS idx_kb_audit_event_resource
    ON kb_audit_event (tenant_id, resource_type, resource_id);

CREATE INDEX IF NOT EXISTS idx_kb_audit_event_event_type
    ON kb_audit_event (tenant_id, event_type, created_at DESC);
