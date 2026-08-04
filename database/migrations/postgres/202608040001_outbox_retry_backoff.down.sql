-- sdkwork:migration
-- id: 202608040001_outbox_retry_backoff
-- engine: postgres
-- module: knowledgebase
-- purpose: Rollback for the outbox retry backoff column and scan index
-- reversible: true
-- transactional: true

SET LOCAL lock_timeout = '2s';
SET LOCAL statement_timeout = '2min';

DROP INDEX IF EXISTS idx_kb_outbox_event_scope_status_retry;
ALTER TABLE kb_outbox_event DROP COLUMN IF EXISTS next_attempt_at;
