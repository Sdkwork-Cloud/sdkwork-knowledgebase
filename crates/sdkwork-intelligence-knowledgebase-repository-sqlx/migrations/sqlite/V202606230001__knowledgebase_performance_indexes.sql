-- Performance indexes for ingestion job polling and outbox stale-claim release.

CREATE INDEX IF NOT EXISTS idx_kb_ingestion_job_tenant_state_status
    ON kb_ingestion_job (tenant_id, state, status);

CREATE INDEX IF NOT EXISTS idx_kb_outbox_stale_claim
    ON kb_outbox_event (tenant_id, status, claimed_at);
