ALTER TABLE kb_outbox_event ADD COLUMN claim_owner TEXT;
ALTER TABLE kb_outbox_event ADD COLUMN claim_token TEXT;
ALTER TABLE kb_outbox_event ADD COLUMN dead_lettered_at TEXT;

-- Reset only stale in-flight claims (claimed longer than the five-minute stale
-- window). Claims still owned by an active worker are preserved so a
-- mid-delivery worker is never silently fenced into duplicate delivery
CREATE INDEX IF NOT EXISTS idx_kb_outbox_event_scope_claim
    ON kb_outbox_event (tenant_id, organization_id, status, claimed_at, id);

UPDATE kb_outbox_event
SET status = 0, claimed_at = NULL, claim_owner = NULL, claim_token = NULL
WHERE status = 3
  AND claimed_at IS NOT NULL
  AND substr(claimed_at, 1, 19) < strftime('%Y-%m-%dT%H:%M:%S', 'now', '-5 minutes');
