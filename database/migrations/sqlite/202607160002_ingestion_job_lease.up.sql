ALTER TABLE kb_ingestion_job ADD COLUMN claim_owner TEXT;
ALTER TABLE kb_ingestion_job ADD COLUMN claim_token TEXT;
ALTER TABLE kb_ingestion_job ADD COLUMN lease_expires_at TEXT;
ALTER TABLE kb_ingestion_job ADD COLUMN attempt_count INTEGER NOT NULL DEFAULT 0;

CREATE INDEX IF NOT EXISTS idx_kb_ingestion_job_claimable
    ON kb_ingestion_job (
        tenant_id,
        status,
        job_type,
        state,
        lease_expires_at,
        priority DESC,
        id
    );
