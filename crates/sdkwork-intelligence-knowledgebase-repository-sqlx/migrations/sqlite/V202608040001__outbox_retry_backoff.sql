-- Exponential retry backoff for outbox delivery failures so dead webhooks are
-- not hammered every poll interval. Exhausted deliveries dead-letter explicitly
ALTER TABLE kb_outbox_event ADD COLUMN next_attempt_at TEXT;

CREATE INDEX IF NOT EXISTS idx_kb_outbox_event_scope_status_retry
    ON kb_outbox_event (tenant_id, organization_id, status, next_attempt_at);
