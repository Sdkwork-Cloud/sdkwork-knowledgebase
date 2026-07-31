CREATE TRIGGER IF NOT EXISTS trg_kb_embedding_scope_insert
BEFORE INSERT ON kb_embedding
WHEN NOT EXISTS (
    SELECT 1 FROM kb_index parent
    WHERE parent.tenant_id = NEW.tenant_id
      AND parent.organization_id = NEW.organization_id
      AND parent.id = NEW.index_id
) OR NOT EXISTS (
    SELECT 1 FROM kb_chunk parent
    WHERE parent.tenant_id = NEW.tenant_id
      AND parent.organization_id = NEW.organization_id
      AND parent.id = NEW.chunk_id
)
BEGIN
    SELECT RAISE(ABORT, 'embedding parent scope mismatch');
END;

CREATE TRIGGER IF NOT EXISTS trg_kb_embedding_scope_update
BEFORE UPDATE OF tenant_id, organization_id, index_id, chunk_id ON kb_embedding
WHEN NOT EXISTS (
    SELECT 1 FROM kb_index parent
    WHERE parent.tenant_id = NEW.tenant_id
      AND parent.organization_id = NEW.organization_id
      AND parent.id = NEW.index_id
) OR NOT EXISTS (
    SELECT 1 FROM kb_chunk parent
    WHERE parent.tenant_id = NEW.tenant_id
      AND parent.organization_id = NEW.organization_id
      AND parent.id = NEW.chunk_id
)
BEGIN
    SELECT RAISE(ABORT, 'embedding parent scope mismatch');
END;
