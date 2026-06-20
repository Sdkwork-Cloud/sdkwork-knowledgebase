-- Knowledge access mode defaults and embedding vector storage for claw-router RAG.

ALTER TABLE kb_agent_profile
    ADD COLUMN knowledge_mode TEXT NOT NULL DEFAULT 'okf_bundle';

ALTER TABLE kb_space
    ADD COLUMN knowledge_mode TEXT NOT NULL DEFAULT 'okf_bundle';

ALTER TABLE kb_embedding
    ADD COLUMN vector_json TEXT;

CREATE INDEX IF NOT EXISTS idx_kb_agent_profile_knowledge_mode
    ON kb_agent_profile (tenant_id, knowledge_mode, status);

CREATE INDEX IF NOT EXISTS idx_kb_space_knowledge_mode
    ON kb_space (tenant_id, knowledge_mode, status);
