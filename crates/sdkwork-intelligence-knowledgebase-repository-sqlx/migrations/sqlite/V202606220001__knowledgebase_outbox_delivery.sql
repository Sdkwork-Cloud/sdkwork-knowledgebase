ALTER TABLE kb_outbox_event ADD COLUMN last_error TEXT;
ALTER TABLE kb_outbox_event ADD COLUMN retry_count INTEGER NOT NULL DEFAULT 0;
