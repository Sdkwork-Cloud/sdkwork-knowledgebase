ALTER TABLE kb_outbox_event ADD COLUMN claim_owner TEXT;
ALTER TABLE kb_outbox_event ADD COLUMN claim_token TEXT;
ALTER TABLE kb_outbox_event ADD COLUMN dead_lettered_at TEXT;

CREATE INDEX IF NOT EXISTS idx_kb_outbox_event_scope_claim
    ON kb_outbox_event (tenant_id, organization_id, status, claimed_at, id);
