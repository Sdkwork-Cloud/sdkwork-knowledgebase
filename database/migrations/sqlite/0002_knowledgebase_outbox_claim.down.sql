-- SQLite 3.35.0+ supports DROP COLUMN. claimed_at is nullable so the rollback is safe.
ALTER TABLE kb_outbox_event DROP COLUMN claimed_at;
