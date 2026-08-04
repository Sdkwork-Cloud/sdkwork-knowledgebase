-- sdkwork:migration
-- id: 202607310002_outbox_claim_fencing
-- engine: postgres
-- module: knowledgebase
-- purpose: Fence outbox claim completion by owner/token and persist exhausted deliveries
-- reversible: false
-- rollback: forward-fix
-- transactional: true
-- lock: heavyweight
-- lock_timeout: 2s
-- statement_timeout: 2min
-- contract_version: 1.3.0
-- rewrite_expectation: nullable metadata columns are metadata-only additions
-- cancellation: cancel before constraint validation and rerun the idempotent migration
-- replication_impact: catalog and index changes only for the pre-launch dataset
-- recovery: restore the pre-migration snapshot or forward-fix invalid claim rows

SET LOCAL lock_timeout = '2s';
SET LOCAL statement_timeout = '2min';

ALTER TABLE kb_outbox_event ADD COLUMN IF NOT EXISTS claim_owner VARCHAR(128);
ALTER TABLE kb_outbox_event ADD COLUMN IF NOT EXISTS claim_token VARCHAR(64);
ALTER TABLE kb_outbox_event ADD COLUMN IF NOT EXISTS dead_lettered_at TIMESTAMP;

-- Reset only stale in-flight claims (claimed longer than the five-minute stale
-- window). Claims still owned by an active worker are preserved so a
-- mid-delivery worker is never silently fenced into duplicate delivery.
-- `claimed_at` is stored as RFC3339 UTC text; the second-precision prefix is
-- compared against the cutoff formatted with the same layout.
UPDATE kb_outbox_event
SET status = 0, claimed_at = NULL, claim_owner = NULL, claim_token = NULL
WHERE status = 3
  AND claimed_at IS NOT NULL
  AND LEFT(claimed_at, 19) < to_char(
        CURRENT_TIMESTAMP - INTERVAL '5 minutes',
        'YYYY-MM-DD"T"HH24:MI:SS'
      );

ALTER TABLE kb_outbox_event DROP CONSTRAINT IF EXISTS ck_kb_outbox_event_claim_pair;
ALTER TABLE kb_outbox_event ADD CONSTRAINT ck_kb_outbox_event_claim_pair CHECK (
    (status = 3 AND claimed_at IS NOT NULL AND claim_owner IS NOT NULL AND claim_token IS NOT NULL)
    OR
    (status <> 3 AND claimed_at IS NULL AND claim_owner IS NULL AND claim_token IS NULL)
) NOT VALID;
ALTER TABLE kb_outbox_event VALIDATE CONSTRAINT ck_kb_outbox_event_claim_pair;

ALTER TABLE kb_outbox_event DROP CONSTRAINT IF EXISTS ck_kb_outbox_event_dead_letter;
ALTER TABLE kb_outbox_event ADD CONSTRAINT ck_kb_outbox_event_dead_letter CHECK (
    (status = 4 AND dead_lettered_at IS NOT NULL)
    OR
    (status <> 4 AND dead_lettered_at IS NULL)
) NOT VALID;
ALTER TABLE kb_outbox_event VALIDATE CONSTRAINT ck_kb_outbox_event_dead_letter;

CREATE INDEX IF NOT EXISTS idx_kb_outbox_event_scope_claim
    ON kb_outbox_event (tenant_id, organization_id, status, claimed_at, id);
CREATE INDEX IF NOT EXISTS idx_kb_outbox_event_scope_dead_letter
    ON kb_outbox_event (tenant_id, organization_id, dead_lettered_at, id)
    WHERE status = 4;
