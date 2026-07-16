ALTER TABLE kb_ingestion_job ADD COLUMN IF NOT EXISTS claim_owner VARCHAR(255);
ALTER TABLE kb_ingestion_job ADD COLUMN IF NOT EXISTS claim_token VARCHAR(64);
ALTER TABLE kb_ingestion_job ADD COLUMN IF NOT EXISTS lease_expires_at TIMESTAMP;
ALTER TABLE kb_ingestion_job ADD COLUMN IF NOT EXISTS attempt_count INTEGER NOT NULL DEFAULT 0;

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
