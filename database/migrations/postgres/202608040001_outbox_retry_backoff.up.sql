-- sdkwork:migration
-- id: 202608040001_outbox_retry_backoff
-- engine: postgres
-- module: knowledgebase
-- purpose: Exponential retry backoff for outbox delivery failures so dead webhooks
--          are not hammered every poll interval; exhausted deliveries dead-letter
--          with an explicit timestamp
-- reversible: false
-- rollback: forward-fix
-- transactional: true
-- lock: medium
-- lock_timeout: 2s
-- statement_timeout: 2min
-- contract_version: 1.3.0
-- rewrite_expectation: nullable metadata column addition only
-- cancellation: cancel and rerun the idempotent migration
-- replication_impact: catalog and index change only
-- recovery: restore the pre-migration snapshot or forward-fix invalid rows

SET LOCAL lock_timeout = '2s';
SET LOCAL statement_timeout = '2min';

ALTER TABLE kb_outbox_event ADD COLUMN IF NOT EXISTS next_attempt_at TIMESTAMP;

-- Serve the worker's requeue scan (status + due-time predicate).
CREATE INDEX IF NOT EXISTS idx_kb_outbox_event_scope_status_retry
    ON kb_outbox_event (tenant_id, organization_id, status, next_attempt_at);
