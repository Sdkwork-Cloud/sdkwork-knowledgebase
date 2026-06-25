ALTER TABLE kb_chunk ADD COLUMN IF NOT EXISTS search_vector tsvector;

UPDATE kb_chunk
SET search_vector = to_tsvector('simple', coalesce(content_text, ''))
WHERE search_vector IS NULL;

CREATE INDEX IF NOT EXISTS idx_kb_chunk_search_vector
    ON kb_chunk USING GIN (search_vector);
