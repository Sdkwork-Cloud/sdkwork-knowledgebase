-- Client-local/test schema mirrors organization ownership needed by repository contracts.
-- Server authority remains the application-root PostgreSQL migration lifecycle.
ALTER TABLE kb_collection ADD COLUMN organization_id INTEGER NOT NULL DEFAULT 0;
ALTER TABLE kb_source ADD COLUMN organization_id INTEGER NOT NULL DEFAULT 0;
ALTER TABLE kb_drive_object_ref ADD COLUMN organization_id INTEGER NOT NULL DEFAULT 0;
ALTER TABLE kb_document ADD COLUMN organization_id INTEGER NOT NULL DEFAULT 0;
ALTER TABLE kb_document_version ADD COLUMN organization_id INTEGER NOT NULL DEFAULT 0;
ALTER TABLE kb_chunk ADD COLUMN organization_id INTEGER NOT NULL DEFAULT 0;
ALTER TABLE kb_index ADD COLUMN organization_id INTEGER NOT NULL DEFAULT 0;
ALTER TABLE kb_embedding ADD COLUMN organization_id INTEGER NOT NULL DEFAULT 0;
ALTER TABLE kb_retrieval_profile ADD COLUMN organization_id INTEGER NOT NULL DEFAULT 0;
ALTER TABLE kb_retrieval_trace ADD COLUMN organization_id INTEGER NOT NULL DEFAULT 0;
ALTER TABLE kb_retrieval_hit ADD COLUMN organization_id INTEGER NOT NULL DEFAULT 0;
ALTER TABLE kb_agent_profile ADD COLUMN organization_id INTEGER NOT NULL DEFAULT 0;
ALTER TABLE kb_agent_knowledge_binding ADD COLUMN organization_id INTEGER NOT NULL DEFAULT 0;
ALTER TABLE kb_ingestion_job ADD COLUMN organization_id INTEGER NOT NULL DEFAULT 0;
ALTER TABLE kb_ingestion_job_item ADD COLUMN organization_id INTEGER NOT NULL DEFAULT 0;
ALTER TABLE kb_okf_concept ADD COLUMN organization_id INTEGER NOT NULL DEFAULT 0;
ALTER TABLE kb_okf_concept_revision ADD COLUMN organization_id INTEGER NOT NULL DEFAULT 0;
ALTER TABLE kb_okf_bundle_file ADD COLUMN organization_id INTEGER NOT NULL DEFAULT 0;
ALTER TABLE kb_okf_schema_profile ADD COLUMN organization_id INTEGER NOT NULL DEFAULT 0;
ALTER TABLE kb_okf_log_entry ADD COLUMN organization_id INTEGER NOT NULL DEFAULT 0;
ALTER TABLE kb_local_mirror_package ADD COLUMN organization_id INTEGER NOT NULL DEFAULT 0;
ALTER TABLE kb_space_context_binding ADD COLUMN organization_id INTEGER NOT NULL DEFAULT 0;
ALTER TABLE kb_outbox_event ADD COLUMN organization_id INTEGER NOT NULL DEFAULT 0;
ALTER TABLE kb_okf_concept_link ADD COLUMN organization_id INTEGER NOT NULL DEFAULT 0;
ALTER TABLE kb_okf_candidate ADD COLUMN organization_id INTEGER NOT NULL DEFAULT 0;
ALTER TABLE kb_market_listing ADD COLUMN organization_id INTEGER NOT NULL DEFAULT 0;
ALTER TABLE kb_market_subscription ADD COLUMN organization_id INTEGER NOT NULL DEFAULT 0;
ALTER TABLE kb_audit_event ADD COLUMN organization_id INTEGER NOT NULL DEFAULT 0;

-- Backfill organization ownership from the parent kb_space so pre-existing
-- fixture rows are never silently re-homed to organization 0 (mirrors the
-- application-root PostgreSQL migration's backfill semantics).
UPDATE kb_collection SET organization_id = COALESCE((SELECT organization_id FROM kb_space WHERE kb_space.id = kb_collection.space_id), organization_id) WHERE organization_id = 0;
UPDATE kb_source SET organization_id = COALESCE((SELECT organization_id FROM kb_space WHERE kb_space.id = kb_source.space_id), organization_id) WHERE organization_id = 0;
UPDATE kb_drive_object_ref SET organization_id = COALESCE((SELECT organization_id FROM kb_space WHERE kb_space.id = kb_drive_object_ref.space_id), organization_id) WHERE organization_id = 0;
UPDATE kb_document SET organization_id = COALESCE((SELECT organization_id FROM kb_space WHERE kb_space.id = kb_document.space_id), organization_id) WHERE organization_id = 0;
UPDATE kb_chunk SET organization_id = COALESCE((SELECT organization_id FROM kb_space WHERE kb_space.id = kb_chunk.space_id), organization_id) WHERE organization_id = 0;
UPDATE kb_index SET organization_id = COALESCE((SELECT organization_id FROM kb_space WHERE kb_space.id = kb_index.space_id), organization_id) WHERE organization_id = 0;
UPDATE kb_agent_knowledge_binding SET organization_id = COALESCE((SELECT organization_id FROM kb_space WHERE kb_space.id = kb_agent_knowledge_binding.space_id), organization_id) WHERE organization_id = 0;
UPDATE kb_ingestion_job SET organization_id = COALESCE((SELECT organization_id FROM kb_space WHERE kb_space.id = kb_ingestion_job.space_id), organization_id) WHERE organization_id = 0;
UPDATE kb_okf_concept SET organization_id = COALESCE((SELECT organization_id FROM kb_space WHERE kb_space.id = kb_okf_concept.space_id), organization_id) WHERE organization_id = 0;
UPDATE kb_okf_bundle_file SET organization_id = COALESCE((SELECT organization_id FROM kb_space WHERE kb_space.id = kb_okf_bundle_file.space_id), organization_id) WHERE organization_id = 0;
UPDATE kb_okf_schema_profile SET organization_id = COALESCE((SELECT organization_id FROM kb_space WHERE kb_space.id = kb_okf_schema_profile.space_id), organization_id) WHERE organization_id = 0;
UPDATE kb_okf_log_entry SET organization_id = COALESCE((SELECT organization_id FROM kb_space WHERE kb_space.id = kb_okf_log_entry.space_id), organization_id) WHERE organization_id = 0;
UPDATE kb_local_mirror_package SET organization_id = COALESCE((SELECT organization_id FROM kb_space WHERE kb_space.id = kb_local_mirror_package.space_id), organization_id) WHERE organization_id = 0;
UPDATE kb_space_context_binding SET organization_id = COALESCE((SELECT organization_id FROM kb_space WHERE kb_space.id = kb_space_context_binding.space_id), organization_id) WHERE organization_id = 0;
UPDATE kb_okf_concept_link SET organization_id = COALESCE((SELECT organization_id FROM kb_space WHERE kb_space.id = kb_okf_concept_link.space_id), organization_id) WHERE organization_id = 0;
UPDATE kb_okf_candidate SET organization_id = COALESCE((SELECT organization_id FROM kb_space WHERE kb_space.id = kb_okf_candidate.space_id), organization_id) WHERE organization_id = 0;
UPDATE kb_market_listing SET organization_id = COALESCE((SELECT organization_id FROM kb_space WHERE kb_space.id = kb_market_listing.space_id), organization_id) WHERE organization_id = 0;
UPDATE kb_document_version SET organization_id = COALESCE((SELECT organization_id FROM kb_document WHERE kb_document.id = kb_document_version.document_id), organization_id) WHERE organization_id = 0;
UPDATE kb_embedding SET organization_id = COALESCE((SELECT organization_id FROM kb_index WHERE kb_index.id = kb_embedding.index_id), organization_id) WHERE organization_id = 0;
UPDATE kb_retrieval_hit SET organization_id = COALESCE((SELECT organization_id FROM kb_chunk WHERE kb_chunk.id = kb_retrieval_hit.chunk_id), organization_id) WHERE organization_id = 0;
UPDATE kb_ingestion_job_item SET organization_id = COALESCE((SELECT organization_id FROM kb_ingestion_job WHERE kb_ingestion_job.id = kb_ingestion_job_item.job_id), organization_id) WHERE organization_id = 0;
UPDATE kb_okf_concept_revision SET organization_id = COALESCE((SELECT organization_id FROM kb_okf_concept WHERE kb_okf_concept.id = kb_okf_concept_revision.concept_row_id), organization_id) WHERE organization_id = 0;
UPDATE kb_market_subscription SET organization_id = COALESCE((SELECT organization_id FROM kb_market_listing WHERE kb_market_listing.id = kb_market_subscription.listing_id), organization_id) WHERE organization_id = 0;

-- FTS is a rebuildable client-local projection. Rebuild it so account or
-- organization switching cannot reuse a tenant-only search index.
DROP TABLE IF EXISTS kb_chunk_fts;
CREATE VIRTUAL TABLE kb_chunk_fts USING fts5(
    content_text,
    chunk_id UNINDEXED,
    tenant_id UNINDEXED,
    organization_id UNINDEXED,
    space_id UNINDEXED,
    document_id UNINDEXED,
    tokenize = 'unicode61'
);
INSERT INTO kb_chunk_fts (
    content_text, chunk_id, tenant_id, organization_id, space_id, document_id
)
SELECT
    content_text, id, tenant_id, organization_id, space_id, document_id
FROM kb_chunk
WHERE status = 1;

DROP INDEX IF EXISTS uk_kb_source_identity;
CREATE UNIQUE INDEX IF NOT EXISTS uk_kb_source_identity
    ON kb_source (
        tenant_id,
        organization_id,
        space_id,
        source_type,
        COALESCE(provider, ''),
        COALESCE(drive_bucket, ''),
        COALESCE(drive_prefix, '')
    )
    WHERE status = 1;

DROP INDEX IF EXISTS uk_kb_space_drive_space;
CREATE UNIQUE INDEX IF NOT EXISTS uk_kb_space_drive_space
    ON kb_space (tenant_id, organization_id, drive_space_id)
    WHERE drive_space_id IS NOT NULL AND status = 1;
DROP INDEX IF EXISTS uk_kb_drive_object_ref_locator;
CREATE UNIQUE INDEX IF NOT EXISTS uk_kb_drive_object_ref_locator
    ON kb_drive_object_ref (
        tenant_id, organization_id, space_id, drive_storage_provider_id,
        drive_bucket, drive_object_key, COALESCE(drive_object_version, ''), object_role
    );
DROP INDEX IF EXISTS uk_kb_document_identity;
CREATE UNIQUE INDEX IF NOT EXISTS uk_kb_document_identity
    ON kb_document (
        tenant_id, organization_id, space_id, collection_id, identity_scope,
        COALESCE(source_id, 0),
        CASE
            WHEN identity_scope = 'source_only' THEN ''
            ELSE COALESCE(original_file_drive_node_id, '')
        END
    )
    WHERE status = 1;
DROP INDEX IF EXISTS uk_kb_document_version_no;
CREATE UNIQUE INDEX IF NOT EXISTS uk_kb_document_version_no
    ON kb_document_version (tenant_id, organization_id, document_id, version_no);
DROP INDEX IF EXISTS uk_kb_chunk_document_version_chunk;
CREATE UNIQUE INDEX IF NOT EXISTS uk_kb_chunk_document_version_chunk
    ON kb_chunk (
        tenant_id, organization_id, document_version_id, chunk_index
    );
DROP INDEX IF EXISTS uk_kb_embedding_index_chunk;
CREATE UNIQUE INDEX IF NOT EXISTS uk_kb_embedding_index_chunk
    ON kb_embedding (tenant_id, organization_id, index_id, chunk_id);
CREATE UNIQUE INDEX IF NOT EXISTS uk_kb_index_active_scope_kind
    ON kb_index (
        tenant_id, organization_id, space_id, collection_id, index_kind
    )
    WHERE status = 1;
DROP INDEX IF EXISTS uk_kb_ingestion_job_idempotency;
CREATE UNIQUE INDEX IF NOT EXISTS uk_kb_ingestion_job_idempotency
    ON kb_ingestion_job (
        tenant_id, organization_id, space_id, idempotency_key
    );
DROP INDEX IF EXISTS uk_kb_okf_concept_id;
CREATE UNIQUE INDEX IF NOT EXISTS uk_kb_okf_concept_id
    ON kb_okf_concept (tenant_id, organization_id, space_id, concept_id);
DROP INDEX IF EXISTS uk_kb_okf_concept_path;
CREATE UNIQUE INDEX IF NOT EXISTS uk_kb_okf_concept_path
    ON kb_okf_concept (tenant_id, organization_id, space_id, logical_path);
DROP INDEX IF EXISTS uk_kb_okf_concept_revision_no;
CREATE UNIQUE INDEX IF NOT EXISTS uk_kb_okf_concept_revision_no
    ON kb_okf_concept_revision (
        tenant_id, organization_id, concept_row_id, revision_no
    );
DROP INDEX IF EXISTS uk_kb_okf_bundle_file_path;
CREATE UNIQUE INDEX IF NOT EXISTS uk_kb_okf_bundle_file_path
    ON kb_okf_bundle_file (tenant_id, organization_id, space_id, logical_path);
DROP INDEX IF EXISTS uk_kb_okf_log_entry_sequence;
CREATE UNIQUE INDEX IF NOT EXISTS uk_kb_okf_log_entry_sequence
    ON kb_okf_log_entry (tenant_id, organization_id, space_id, sequence_no);
DROP INDEX IF EXISTS uk_kb_space_context;
CREATE UNIQUE INDEX IF NOT EXISTS uk_kb_space_context
    ON kb_space_context_binding (
        tenant_id, organization_id, space_id, context_type, context_id
    )
    WHERE status = 1;
DROP INDEX IF EXISTS uk_kb_okf_concept_link_edge;
CREATE UNIQUE INDEX IF NOT EXISTS uk_kb_okf_concept_link_edge
    ON kb_okf_concept_link (
        tenant_id, organization_id, space_id,
        from_concept_id, to_concept_id, anchor_text
    );
DROP INDEX IF EXISTS uk_kb_market_listing_space;
CREATE UNIQUE INDEX IF NOT EXISTS uk_kb_market_listing_space
    ON kb_market_listing (tenant_id, organization_id, space_id)
    WHERE status = 1;
DROP INDEX IF EXISTS uk_kb_market_subscription_actor_listing;
CREATE UNIQUE INDEX IF NOT EXISTS uk_kb_market_subscription_actor_listing
    ON kb_market_subscription (
        tenant_id, organization_id, subscriber_actor_id, listing_id
    )
    WHERE status = 1;

CREATE INDEX IF NOT EXISTS idx_kb_source_scope_active
    ON kb_source (tenant_id, organization_id, status, id);
CREATE INDEX IF NOT EXISTS idx_kb_chunk_scope_search
    ON kb_chunk (tenant_id, organization_id, space_id, collection_id, status, id);
CREATE INDEX IF NOT EXISTS idx_kb_embedding_scope_chunk
    ON kb_embedding (tenant_id, organization_id, chunk_id, status);
CREATE INDEX IF NOT EXISTS idx_kb_retrieval_trace_scope_id
    ON kb_retrieval_trace (tenant_id, organization_id, id);
CREATE INDEX IF NOT EXISTS idx_kb_retrieval_hit_scope_trace_rank
    ON kb_retrieval_hit (
        tenant_id, organization_id, retrieval_trace_id, result_rank, id
    );

CREATE INDEX IF NOT EXISTS idx_kb_outbox_event_scope_status_created
    ON kb_outbox_event (tenant_id, organization_id, status, created_at, id);
CREATE INDEX IF NOT EXISTS idx_kb_audit_event_scope_actor_created
    ON kb_audit_event (tenant_id, organization_id, actor_id, created_at DESC, id DESC);
