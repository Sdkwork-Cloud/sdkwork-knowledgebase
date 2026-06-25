ALTER TABLE kb_outbox_event ADD COLUMN IF NOT EXISTS claimed_at TEXT;
