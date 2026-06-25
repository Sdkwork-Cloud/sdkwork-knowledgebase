CREATE VIRTUAL TABLE IF NOT EXISTS kb_chunk_fts USING fts5(
    content_text,
    chunk_id UNINDEXED,
    tenant_id UNINDEXED,
    space_id UNINDEXED,
    document_id UNINDEXED,
    tokenize = 'unicode61'
);

INSERT INTO kb_chunk_fts (content_text, chunk_id, tenant_id, space_id, document_id)
SELECT c.content_text, c.id, c.tenant_id, c.space_id, c.document_id
FROM kb_chunk c
WHERE c.status = 1;
