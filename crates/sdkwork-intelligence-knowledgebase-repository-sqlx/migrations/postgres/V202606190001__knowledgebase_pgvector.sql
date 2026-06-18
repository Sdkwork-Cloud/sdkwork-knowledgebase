-- PostgreSQL pgvector extension and ANN-ready embedding column (forward-compatible).
CREATE EXTENSION IF NOT EXISTS vector;

ALTER TABLE kb_embedding
    ADD COLUMN IF NOT EXISTS embedding_vector vector(1536);

CREATE INDEX IF NOT EXISTS idx_kb_embedding_vector_hnsw
    ON kb_embedding
    USING hnsw (embedding_vector vector_cosine_ops)
    WHERE embedding_vector IS NOT NULL AND status = 1;
