-- Configurable agent runtime implementation (Rig is the default).

ALTER TABLE kb_agent_profile
    ADD COLUMN agent_implementation_id TEXT NOT NULL DEFAULT 'plugin.intelligence.rig';

CREATE INDEX IF NOT EXISTS idx_kb_agent_profile_agent_implementation
    ON kb_agent_profile (tenant_id, agent_implementation_id, status);
