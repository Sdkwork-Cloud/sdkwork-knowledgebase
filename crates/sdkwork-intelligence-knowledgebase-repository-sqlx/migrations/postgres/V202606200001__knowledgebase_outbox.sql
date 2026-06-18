CREATE TABLE IF NOT EXISTS kb_outbox_event (
    id BIGINT PRIMARY KEY,
    uuid VARCHAR(64) NOT NULL,
    tenant_id BIGINT NOT NULL,
    aggregate_type VARCHAR(64) NOT NULL,
    aggregate_id BIGINT NOT NULL,
    event_type VARCHAR(128) NOT NULL,
    payload JSONB NOT NULL,
    status INTEGER NOT NULL,
    created_at TIMESTAMP NOT NULL,
    published_at TIMESTAMP,
    version BIGINT NOT NULL DEFAULT 0
);

CREATE UNIQUE INDEX IF NOT EXISTS uk_kb_outbox_event_uuid
    ON kb_outbox_event (tenant_id, uuid);

CREATE INDEX IF NOT EXISTS idx_kb_outbox_event_status_created
    ON kb_outbox_event (tenant_id, status, created_at);
