CREATE TABLE IF NOT EXISTS kb_outbox_event (
    id INTEGER NOT NULL,
    uuid TEXT NOT NULL,
    tenant_id INTEGER NOT NULL,
    aggregate_type TEXT NOT NULL,
    aggregate_id INTEGER NOT NULL,
    event_type TEXT NOT NULL,
    payload TEXT NOT NULL,
    status INTEGER NOT NULL,
    created_at TEXT NOT NULL,
    published_at TEXT,
    version INTEGER NOT NULL DEFAULT 0,
    PRIMARY KEY (id)
);

CREATE UNIQUE INDEX IF NOT EXISTS uk_kb_outbox_event_uuid
    ON kb_outbox_event (tenant_id, uuid);

CREATE INDEX IF NOT EXISTS idx_kb_outbox_event_status_created
    ON kb_outbox_event (tenant_id, status, created_at);
