-- sdkwork:migration
-- id: 202608040002_audit_event_scope_actor_index
-- engine: postgres
-- module: knowledgebase
-- purpose: Composite (tenant, organization, actor, created_at) index serving the
--          backend audit-event cursor list; SQLite already carries the matching
--          index, PostgreSQL did not
-- reversible: false
-- rollback: forward-fix
-- transactional: true
-- lock: medium
-- lock_timeout: 2s
-- statement_timeout: 2min
-- contract_version: 1.3.0
-- rewrite_expectation: index addition only
-- cancellation: cancel and rerun the idempotent migration
-- replication_impact: index creation only
-- recovery: restore the pre-migration snapshot or re-create the index

SET LOCAL lock_timeout = '2s';
SET LOCAL statement_timeout = '2min';

CREATE INDEX IF NOT EXISTS idx_kb_audit_event_scope_actor_created
    ON kb_audit_event (tenant_id, organization_id, actor_id, created_at DESC, id DESC);
