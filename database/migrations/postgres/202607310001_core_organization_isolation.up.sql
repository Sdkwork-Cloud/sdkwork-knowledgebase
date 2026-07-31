-- sdkwork:migration
-- id: 202607310001_core_organization_isolation
-- engine: postgres
-- module: knowledgebase
-- purpose: Add first-class organization scope, backfill it through knowledge spaces, and enforce organization RLS
-- reversible: false
-- rollback: forward-fix
-- transactional: true
-- lock: heavyweight
-- lock_timeout: 2s
-- statement_timeout: 5min
-- contract_version: 1.2.0
-- rewrite_expectation: nullable column expansion is metadata-only; backfill updates existing rows in bounded pre-launch datasets
-- cancellation: cancel before constraint validation and rerun the idempotent migration after resolving orphaned rows
-- replication_impact: row updates generate WAL proportional to the pre-launch dataset; release gate requires zero production data
-- recovery: restore the pre-migration snapshot or forward-fix orphaned ownership and rerun

SET LOCAL lock_timeout = '2s';
SET LOCAL statement_timeout = '5min';

ALTER TABLE kb_space ALTER COLUMN organization_id DROP DEFAULT;

ALTER TABLE kb_collection ADD COLUMN IF NOT EXISTS organization_id BIGINT;
ALTER TABLE kb_source ADD COLUMN IF NOT EXISTS organization_id BIGINT;
ALTER TABLE kb_drive_object_ref ADD COLUMN IF NOT EXISTS organization_id BIGINT;
ALTER TABLE kb_document ADD COLUMN IF NOT EXISTS organization_id BIGINT;
ALTER TABLE kb_document_version ADD COLUMN IF NOT EXISTS organization_id BIGINT;
ALTER TABLE kb_chunk ADD COLUMN IF NOT EXISTS organization_id BIGINT;
ALTER TABLE kb_index ADD COLUMN IF NOT EXISTS organization_id BIGINT;
ALTER TABLE kb_embedding ADD COLUMN IF NOT EXISTS organization_id BIGINT;
ALTER TABLE kb_retrieval_profile ADD COLUMN IF NOT EXISTS organization_id BIGINT;
ALTER TABLE kb_retrieval_trace ADD COLUMN IF NOT EXISTS organization_id BIGINT;
ALTER TABLE kb_retrieval_hit ADD COLUMN IF NOT EXISTS organization_id BIGINT;
ALTER TABLE kb_agent_profile ADD COLUMN IF NOT EXISTS organization_id BIGINT;
ALTER TABLE kb_agent_knowledge_binding ADD COLUMN IF NOT EXISTS organization_id BIGINT;
ALTER TABLE kb_ingestion_job ADD COLUMN IF NOT EXISTS organization_id BIGINT;
ALTER TABLE kb_ingestion_job_item ADD COLUMN IF NOT EXISTS organization_id BIGINT;
ALTER TABLE kb_okf_concept ADD COLUMN IF NOT EXISTS organization_id BIGINT;
ALTER TABLE kb_okf_concept_revision ADD COLUMN IF NOT EXISTS organization_id BIGINT;
ALTER TABLE kb_okf_bundle_file ADD COLUMN IF NOT EXISTS organization_id BIGINT;
ALTER TABLE kb_okf_schema_profile ADD COLUMN IF NOT EXISTS organization_id BIGINT;
ALTER TABLE kb_okf_log_entry ADD COLUMN IF NOT EXISTS organization_id BIGINT;
ALTER TABLE kb_local_mirror_package ADD COLUMN IF NOT EXISTS organization_id BIGINT;
ALTER TABLE kb_space_context_binding ADD COLUMN IF NOT EXISTS organization_id BIGINT;
ALTER TABLE kb_outbox_event ADD COLUMN IF NOT EXISTS organization_id BIGINT;
ALTER TABLE kb_okf_concept_link ADD COLUMN IF NOT EXISTS organization_id BIGINT;
ALTER TABLE kb_okf_candidate ADD COLUMN IF NOT EXISTS organization_id BIGINT;
ALTER TABLE kb_market_listing ADD COLUMN IF NOT EXISTS organization_id BIGINT;
ALTER TABLE kb_market_subscription ADD COLUMN IF NOT EXISTS organization_id BIGINT;
ALTER TABLE kb_audit_event ADD COLUMN IF NOT EXISTS organization_id BIGINT;

UPDATE kb_collection target SET organization_id = space.organization_id
FROM kb_space space
WHERE target.organization_id IS NULL AND target.tenant_id = space.tenant_id AND target.space_id = space.id;
UPDATE kb_source target SET organization_id = space.organization_id
FROM kb_space space
WHERE target.organization_id IS NULL AND target.tenant_id = space.tenant_id AND target.space_id = space.id;
UPDATE kb_drive_object_ref target SET organization_id = space.organization_id
FROM kb_space space
WHERE target.organization_id IS NULL AND target.tenant_id = space.tenant_id AND target.space_id = space.id;
UPDATE kb_document target SET organization_id = space.organization_id
FROM kb_space space
WHERE target.organization_id IS NULL AND target.tenant_id = space.tenant_id AND target.space_id = space.id;
UPDATE kb_chunk target SET organization_id = space.organization_id
FROM kb_space space
WHERE target.organization_id IS NULL AND target.tenant_id = space.tenant_id AND target.space_id = space.id;
UPDATE kb_index target SET organization_id = space.organization_id
FROM kb_space space
WHERE target.organization_id IS NULL AND target.tenant_id = space.tenant_id AND target.space_id = space.id;
UPDATE kb_agent_knowledge_binding target SET organization_id = space.organization_id
FROM kb_space space
WHERE target.organization_id IS NULL AND target.tenant_id = space.tenant_id AND target.space_id = space.id;
UPDATE kb_ingestion_job target SET organization_id = space.organization_id
FROM kb_space space
WHERE target.organization_id IS NULL AND target.tenant_id = space.tenant_id AND target.space_id = space.id;
UPDATE kb_okf_concept target SET organization_id = space.organization_id
FROM kb_space space
WHERE target.organization_id IS NULL AND target.tenant_id = space.tenant_id AND target.space_id = space.id;
UPDATE kb_okf_bundle_file target SET organization_id = space.organization_id
FROM kb_space space
WHERE target.organization_id IS NULL AND target.tenant_id = space.tenant_id AND target.space_id = space.id;
UPDATE kb_okf_schema_profile target SET organization_id = space.organization_id
FROM kb_space space
WHERE target.organization_id IS NULL AND target.tenant_id = space.tenant_id AND target.space_id = space.id;
UPDATE kb_okf_log_entry target SET organization_id = space.organization_id
FROM kb_space space
WHERE target.organization_id IS NULL AND target.tenant_id = space.tenant_id AND target.space_id = space.id;
UPDATE kb_local_mirror_package target SET organization_id = space.organization_id
FROM kb_space space
WHERE target.organization_id IS NULL AND target.tenant_id = space.tenant_id AND target.space_id = space.id;
UPDATE kb_space_context_binding target SET organization_id = space.organization_id
FROM kb_space space
WHERE target.organization_id IS NULL AND target.tenant_id = space.tenant_id AND target.space_id = space.id;
UPDATE kb_okf_concept_link target SET organization_id = space.organization_id
FROM kb_space space
WHERE target.organization_id IS NULL AND target.tenant_id = space.tenant_id AND target.space_id = space.id;
UPDATE kb_okf_candidate target SET organization_id = space.organization_id
FROM kb_space space
WHERE target.organization_id IS NULL AND target.tenant_id = space.tenant_id AND target.space_id = space.id;
UPDATE kb_market_listing target SET organization_id = space.organization_id
FROM kb_space space
WHERE target.organization_id IS NULL AND target.tenant_id = space.tenant_id AND target.space_id = space.id;

UPDATE kb_document_version target SET organization_id = document.organization_id
FROM kb_document document
WHERE target.organization_id IS NULL AND target.tenant_id = document.tenant_id AND target.document_id = document.id;
UPDATE kb_embedding target SET organization_id = knowledge_index.organization_id
FROM kb_index knowledge_index
WHERE target.organization_id IS NULL AND target.tenant_id = knowledge_index.tenant_id AND target.index_id = knowledge_index.id;
UPDATE kb_retrieval_hit target SET organization_id = chunk.organization_id
FROM kb_chunk chunk
WHERE target.organization_id IS NULL AND target.tenant_id = chunk.tenant_id AND target.chunk_id = chunk.id;
UPDATE kb_ingestion_job_item target SET organization_id = job.organization_id
FROM kb_ingestion_job job
WHERE target.organization_id IS NULL AND target.tenant_id = job.tenant_id AND target.job_id = job.id;
UPDATE kb_okf_concept_revision target SET organization_id = concept.organization_id
FROM kb_okf_concept concept
WHERE target.organization_id IS NULL AND target.tenant_id = concept.tenant_id AND target.concept_row_id = concept.id;
UPDATE kb_market_subscription target SET organization_id = listing.organization_id
FROM kb_market_listing listing
WHERE target.organization_id IS NULL AND target.tenant_id = listing.tenant_id AND target.listing_id = listing.id;

DO $$
BEGIN
    IF EXISTS (
        SELECT 1 FROM kb_agent_knowledge_binding
        GROUP BY tenant_id, profile_id HAVING COUNT(DISTINCT organization_id) > 1
    ) THEN
        RAISE EXCEPTION 'agent profile is bound across organizations; split the profile before organization cutover';
    END IF;
END $$;

UPDATE kb_agent_profile target SET organization_id = scope.organization_id
FROM (
    SELECT tenant_id, profile_id, MIN(organization_id) AS organization_id
    FROM kb_agent_knowledge_binding GROUP BY tenant_id, profile_id
) scope
WHERE target.organization_id IS NULL AND target.tenant_id = scope.tenant_id AND target.id = scope.profile_id;
UPDATE kb_agent_profile SET organization_id = 0 WHERE organization_id IS NULL;

DO $$
BEGIN
    IF EXISTS (
        SELECT 1 FROM kb_agent_profile WHERE retrieval_profile_id IS NOT NULL
        GROUP BY tenant_id, retrieval_profile_id HAVING COUNT(DISTINCT organization_id) > 1
    ) THEN
        RAISE EXCEPTION 'retrieval profile is referenced across organizations; split the profile before organization cutover';
    END IF;
END $$;

UPDATE kb_retrieval_profile target SET organization_id = scope.organization_id
FROM (
    SELECT tenant_id, retrieval_profile_id, MIN(organization_id) AS organization_id
    FROM kb_agent_profile WHERE retrieval_profile_id IS NOT NULL
    GROUP BY tenant_id, retrieval_profile_id
) scope
WHERE target.organization_id IS NULL AND target.tenant_id = scope.tenant_id AND target.id = scope.retrieval_profile_id;
UPDATE kb_retrieval_profile SET organization_id = 0 WHERE organization_id IS NULL;

DO $$
BEGIN
    IF EXISTS (
        SELECT 1 FROM kb_retrieval_hit
        GROUP BY tenant_id, retrieval_trace_id HAVING COUNT(DISTINCT organization_id) > 1
    ) THEN
        RAISE EXCEPTION 'retrieval trace contains hits from multiple organizations';
    END IF;
END $$;

UPDATE kb_retrieval_trace target SET organization_id = scope.organization_id
FROM (
    SELECT tenant_id, retrieval_trace_id, MIN(organization_id) AS organization_id
    FROM kb_retrieval_hit GROUP BY tenant_id, retrieval_trace_id
) scope
WHERE target.organization_id IS NULL AND target.tenant_id = scope.tenant_id AND target.id = scope.retrieval_trace_id;
UPDATE kb_retrieval_trace SET organization_id = 0 WHERE organization_id IS NULL;

UPDATE kb_outbox_event target SET organization_id = job.organization_id
FROM kb_ingestion_job job
WHERE target.organization_id IS NULL AND target.aggregate_type = 'ingestion_job'
  AND target.tenant_id = job.tenant_id AND target.aggregate_id = job.id;
UPDATE kb_outbox_event target SET organization_id = document.organization_id
FROM kb_document document
WHERE target.organization_id IS NULL AND target.aggregate_type = 'knowledge_document'
  AND target.tenant_id = document.tenant_id AND target.aggregate_id = document.id;
UPDATE kb_outbox_event target SET organization_id = publication.organization_id
FROM kb_site_publication publication
WHERE target.organization_id IS NULL AND target.aggregate_type = 'wiki_publication'
  AND target.tenant_id = publication.tenant_id AND target.aggregate_id = publication.id;
UPDATE kb_outbox_event SET organization_id = 0 WHERE organization_id IS NULL;
UPDATE kb_audit_event SET organization_id = 0 WHERE organization_id IS NULL;

DO $$
DECLARE
    table_name text;
    missing_count bigint;
BEGIN
    FOR table_name IN SELECT unnest(ARRAY[
        'kb_collection', 'kb_source', 'kb_drive_object_ref', 'kb_document',
        'kb_document_version', 'kb_chunk', 'kb_index', 'kb_embedding',
        'kb_retrieval_profile', 'kb_retrieval_trace', 'kb_retrieval_hit',
        'kb_agent_profile', 'kb_agent_knowledge_binding', 'kb_ingestion_job',
        'kb_ingestion_job_item', 'kb_okf_concept', 'kb_okf_concept_revision',
        'kb_okf_bundle_file', 'kb_okf_schema_profile', 'kb_okf_log_entry',
        'kb_local_mirror_package', 'kb_space_context_binding', 'kb_outbox_event',
        'kb_okf_concept_link', 'kb_okf_candidate', 'kb_market_listing',
        'kb_market_subscription', 'kb_audit_event'
    ])
    LOOP
        EXECUTE format('SELECT COUNT(*) FROM %I WHERE organization_id IS NULL', table_name)
            INTO missing_count;
        IF missing_count > 0 THEN
            RAISE EXCEPTION '% has % rows without an owning organization', table_name, missing_count;
        END IF;
        EXECUTE format('ALTER TABLE %I ALTER COLUMN organization_id SET NOT NULL', table_name);
        EXECUTE format(
            'ALTER TABLE %I ADD CONSTRAINT %I CHECK (organization_id >= 0) NOT VALID',
            table_name,
            'ck_' || table_name || '_organization'
        );
        EXECUTE format(
            'ALTER TABLE %I VALIDATE CONSTRAINT %I',
            table_name,
            'ck_' || table_name || '_organization'
        );
        EXECUTE format(
            'CREATE UNIQUE INDEX IF NOT EXISTS %I ON %I (tenant_id, organization_id, id)',
            'uk_' || table_name || '_scope_id',
            table_name
        );
    END LOOP;
END $$;

ALTER TABLE kb_space DROP CONSTRAINT IF EXISTS ck_kb_space_organization;
ALTER TABLE kb_space ADD CONSTRAINT ck_kb_space_organization CHECK (organization_id >= 0) NOT VALID;
ALTER TABLE kb_space VALIDATE CONSTRAINT ck_kb_space_organization;
CREATE UNIQUE INDEX IF NOT EXISTS uk_kb_space_scope_id ON kb_space (tenant_id, organization_id, id);

DROP INDEX IF EXISTS uk_kb_source_identity;
CREATE UNIQUE INDEX uk_kb_source_identity ON kb_source (
    tenant_id, organization_id, space_id, source_type,
    COALESCE(provider, ''), COALESCE(drive_bucket, ''), COALESCE(drive_prefix, '')
) WHERE status = 1;

DROP INDEX IF EXISTS uk_kb_space_drive_space;
CREATE UNIQUE INDEX uk_kb_space_drive_space
    ON kb_space (tenant_id, organization_id, drive_space_id)
    WHERE drive_space_id IS NOT NULL AND status = 1;
DROP INDEX IF EXISTS uk_kb_drive_object_ref_locator;
CREATE UNIQUE INDEX uk_kb_drive_object_ref_locator ON kb_drive_object_ref (
    tenant_id, organization_id, space_id, drive_storage_provider_id,
    drive_bucket, drive_object_key, COALESCE(drive_object_version, ''), object_role
);
DROP INDEX IF EXISTS uk_kb_document_identity;
CREATE UNIQUE INDEX uk_kb_document_identity ON kb_document (
    tenant_id, organization_id, space_id, collection_id, identity_scope,
    COALESCE(source_id, 0),
    (
        CASE
            WHEN identity_scope = 'source_only' THEN ''
            ELSE COALESCE(original_file_drive_node_id, '')
        END
    )
) WHERE status = 1;
DROP INDEX IF EXISTS uk_kb_document_version_no;
CREATE UNIQUE INDEX uk_kb_document_version_no
    ON kb_document_version (tenant_id, organization_id, document_id, version_no);
DROP INDEX IF EXISTS uk_kb_chunk_document_version_chunk;
CREATE UNIQUE INDEX uk_kb_chunk_document_version_chunk
    ON kb_chunk (tenant_id, organization_id, document_version_id, chunk_index);
DROP INDEX IF EXISTS uk_kb_embedding_index_chunk;
CREATE UNIQUE INDEX uk_kb_embedding_index_chunk
    ON kb_embedding (tenant_id, organization_id, index_id, chunk_id);
CREATE UNIQUE INDEX IF NOT EXISTS uk_kb_index_active_scope_kind
    ON kb_index (
        tenant_id, organization_id, space_id, collection_id, index_kind
    )
    WHERE status = 1;
DROP INDEX IF EXISTS uk_kb_ingestion_job_idempotency;
CREATE UNIQUE INDEX uk_kb_ingestion_job_idempotency
    ON kb_ingestion_job (tenant_id, organization_id, space_id, idempotency_key);
DROP INDEX IF EXISTS uk_kb_okf_concept_id;
CREATE UNIQUE INDEX uk_kb_okf_concept_id
    ON kb_okf_concept (tenant_id, organization_id, space_id, concept_id);
DROP INDEX IF EXISTS uk_kb_okf_concept_path;
CREATE UNIQUE INDEX uk_kb_okf_concept_path
    ON kb_okf_concept (tenant_id, organization_id, space_id, logical_path);
DROP INDEX IF EXISTS uk_kb_okf_concept_revision_no;
CREATE UNIQUE INDEX uk_kb_okf_concept_revision_no
    ON kb_okf_concept_revision (
        tenant_id, organization_id, concept_row_id, revision_no
    );
DROP INDEX IF EXISTS uk_kb_okf_bundle_file_path;
CREATE UNIQUE INDEX uk_kb_okf_bundle_file_path
    ON kb_okf_bundle_file (tenant_id, organization_id, space_id, logical_path);
DROP INDEX IF EXISTS uk_kb_okf_log_entry_sequence;
CREATE UNIQUE INDEX uk_kb_okf_log_entry_sequence
    ON kb_okf_log_entry (tenant_id, organization_id, space_id, sequence_no);
DROP INDEX IF EXISTS uk_kb_space_context;
CREATE UNIQUE INDEX uk_kb_space_context ON kb_space_context_binding (
    tenant_id, organization_id, space_id, context_type, context_id
) WHERE status = 1;
DROP INDEX IF EXISTS uk_kb_okf_concept_link_edge;
CREATE UNIQUE INDEX uk_kb_okf_concept_link_edge ON kb_okf_concept_link (
    tenant_id, organization_id, space_id,
    from_concept_id, to_concept_id, anchor_text
);
DROP INDEX IF EXISTS uk_kb_market_listing_space;
CREATE UNIQUE INDEX uk_kb_market_listing_space
    ON kb_market_listing (tenant_id, organization_id, space_id)
    WHERE status = 1;
DROP INDEX IF EXISTS uk_kb_market_subscription_actor_listing;
CREATE UNIQUE INDEX uk_kb_market_subscription_actor_listing
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

ALTER TABLE kb_collection ADD CONSTRAINT fk_kb_collection_space_scope
    FOREIGN KEY (tenant_id, organization_id, space_id) REFERENCES kb_space(tenant_id, organization_id, id) NOT VALID;
ALTER TABLE kb_source ADD CONSTRAINT fk_kb_source_space_scope
    FOREIGN KEY (tenant_id, organization_id, space_id) REFERENCES kb_space(tenant_id, organization_id, id) NOT VALID;
ALTER TABLE kb_drive_object_ref ADD CONSTRAINT fk_kb_drive_object_ref_space_scope
    FOREIGN KEY (tenant_id, organization_id, space_id) REFERENCES kb_space(tenant_id, organization_id, id) NOT VALID;
ALTER TABLE kb_document ADD CONSTRAINT fk_kb_document_space_scope
    FOREIGN KEY (tenant_id, organization_id, space_id) REFERENCES kb_space(tenant_id, organization_id, id) NOT VALID;
ALTER TABLE kb_document_version ADD CONSTRAINT fk_kb_document_version_document_scope
    FOREIGN KEY (tenant_id, organization_id, document_id) REFERENCES kb_document(tenant_id, organization_id, id) NOT VALID;
ALTER TABLE kb_chunk ADD CONSTRAINT fk_kb_chunk_space_scope
    FOREIGN KEY (tenant_id, organization_id, space_id) REFERENCES kb_space(tenant_id, organization_id, id) NOT VALID;
ALTER TABLE kb_chunk ADD CONSTRAINT fk_kb_chunk_document_scope
    FOREIGN KEY (tenant_id, organization_id, document_id) REFERENCES kb_document(tenant_id, organization_id, id) NOT VALID;
ALTER TABLE kb_index ADD CONSTRAINT fk_kb_index_space_scope
    FOREIGN KEY (tenant_id, organization_id, space_id) REFERENCES kb_space(tenant_id, organization_id, id) NOT VALID;
ALTER TABLE kb_embedding ADD CONSTRAINT fk_kb_embedding_index_scope
    FOREIGN KEY (tenant_id, organization_id, index_id) REFERENCES kb_index(tenant_id, organization_id, id) NOT VALID;
ALTER TABLE kb_embedding ADD CONSTRAINT fk_kb_embedding_chunk_scope
    FOREIGN KEY (tenant_id, organization_id, chunk_id) REFERENCES kb_chunk(tenant_id, organization_id, id) NOT VALID;
ALTER TABLE kb_retrieval_hit ADD CONSTRAINT fk_kb_retrieval_hit_trace_scope
    FOREIGN KEY (tenant_id, organization_id, retrieval_trace_id) REFERENCES kb_retrieval_trace(tenant_id, organization_id, id) NOT VALID;
ALTER TABLE kb_retrieval_hit ADD CONSTRAINT fk_kb_retrieval_hit_chunk_scope
    FOREIGN KEY (tenant_id, organization_id, chunk_id) REFERENCES kb_chunk(tenant_id, organization_id, id) NOT VALID;
ALTER TABLE kb_retrieval_hit ADD CONSTRAINT fk_kb_retrieval_hit_document_scope
    FOREIGN KEY (tenant_id, organization_id, document_id) REFERENCES kb_document(tenant_id, organization_id, id) NOT VALID;
ALTER TABLE kb_retrieval_hit ADD CONSTRAINT fk_kb_retrieval_hit_version_scope
    FOREIGN KEY (tenant_id, organization_id, document_version_id) REFERENCES kb_document_version(tenant_id, organization_id, id) NOT VALID;
ALTER TABLE kb_agent_knowledge_binding ADD CONSTRAINT fk_kb_agent_binding_profile_scope
    FOREIGN KEY (tenant_id, organization_id, profile_id) REFERENCES kb_agent_profile(tenant_id, organization_id, id) NOT VALID;
ALTER TABLE kb_agent_knowledge_binding ADD CONSTRAINT fk_kb_agent_binding_space_scope
    FOREIGN KEY (tenant_id, organization_id, space_id) REFERENCES kb_space(tenant_id, organization_id, id) NOT VALID;
ALTER TABLE kb_ingestion_job ADD CONSTRAINT fk_kb_ingestion_job_space_scope
    FOREIGN KEY (tenant_id, organization_id, space_id) REFERENCES kb_space(tenant_id, organization_id, id) NOT VALID;
ALTER TABLE kb_ingestion_job_item ADD CONSTRAINT fk_kb_ingestion_job_item_job_scope
    FOREIGN KEY (tenant_id, organization_id, job_id) REFERENCES kb_ingestion_job(tenant_id, organization_id, id) NOT VALID;
ALTER TABLE kb_okf_concept ADD CONSTRAINT fk_kb_okf_concept_space_scope
    FOREIGN KEY (tenant_id, organization_id, space_id) REFERENCES kb_space(tenant_id, organization_id, id) NOT VALID;
ALTER TABLE kb_okf_concept_revision ADD CONSTRAINT fk_kb_okf_revision_concept_scope
    FOREIGN KEY (tenant_id, organization_id, concept_row_id) REFERENCES kb_okf_concept(tenant_id, organization_id, id) NOT VALID;
ALTER TABLE kb_market_listing ADD CONSTRAINT fk_kb_market_listing_space_scope
    FOREIGN KEY (tenant_id, organization_id, space_id) REFERENCES kb_space(tenant_id, organization_id, id) NOT VALID;
ALTER TABLE kb_market_subscription ADD CONSTRAINT fk_kb_market_subscription_listing_scope
    FOREIGN KEY (tenant_id, organization_id, listing_id) REFERENCES kb_market_listing(tenant_id, organization_id, id) NOT VALID;

ALTER TABLE kb_collection VALIDATE CONSTRAINT fk_kb_collection_space_scope;
ALTER TABLE kb_source VALIDATE CONSTRAINT fk_kb_source_space_scope;
ALTER TABLE kb_drive_object_ref VALIDATE CONSTRAINT fk_kb_drive_object_ref_space_scope;
ALTER TABLE kb_document VALIDATE CONSTRAINT fk_kb_document_space_scope;
ALTER TABLE kb_document_version VALIDATE CONSTRAINT fk_kb_document_version_document_scope;
ALTER TABLE kb_chunk VALIDATE CONSTRAINT fk_kb_chunk_space_scope;
ALTER TABLE kb_chunk VALIDATE CONSTRAINT fk_kb_chunk_document_scope;
ALTER TABLE kb_index VALIDATE CONSTRAINT fk_kb_index_space_scope;
ALTER TABLE kb_embedding VALIDATE CONSTRAINT fk_kb_embedding_index_scope;
ALTER TABLE kb_embedding VALIDATE CONSTRAINT fk_kb_embedding_chunk_scope;
ALTER TABLE kb_retrieval_hit VALIDATE CONSTRAINT fk_kb_retrieval_hit_trace_scope;
ALTER TABLE kb_retrieval_hit VALIDATE CONSTRAINT fk_kb_retrieval_hit_chunk_scope;
ALTER TABLE kb_retrieval_hit VALIDATE CONSTRAINT fk_kb_retrieval_hit_document_scope;
ALTER TABLE kb_retrieval_hit VALIDATE CONSTRAINT fk_kb_retrieval_hit_version_scope;
ALTER TABLE kb_agent_knowledge_binding VALIDATE CONSTRAINT fk_kb_agent_binding_profile_scope;
ALTER TABLE kb_agent_knowledge_binding VALIDATE CONSTRAINT fk_kb_agent_binding_space_scope;
ALTER TABLE kb_ingestion_job VALIDATE CONSTRAINT fk_kb_ingestion_job_space_scope;
ALTER TABLE kb_ingestion_job_item VALIDATE CONSTRAINT fk_kb_ingestion_job_item_job_scope;
ALTER TABLE kb_okf_concept VALIDATE CONSTRAINT fk_kb_okf_concept_space_scope;
ALTER TABLE kb_okf_concept_revision VALIDATE CONSTRAINT fk_kb_okf_revision_concept_scope;
ALTER TABLE kb_market_listing VALIDATE CONSTRAINT fk_kb_market_listing_space_scope;
ALTER TABLE kb_market_subscription VALIDATE CONSTRAINT fk_kb_market_subscription_listing_scope;

DO $$
DECLARE
    table_name text;
BEGIN
    FOR table_name IN SELECT unnest(ARRAY[
        'kb_space', 'kb_collection', 'kb_source', 'kb_drive_object_ref', 'kb_document',
        'kb_document_version', 'kb_chunk', 'kb_index', 'kb_embedding',
        'kb_retrieval_profile', 'kb_retrieval_trace', 'kb_retrieval_hit',
        'kb_agent_profile', 'kb_agent_knowledge_binding', 'kb_ingestion_job',
        'kb_ingestion_job_item', 'kb_okf_concept', 'kb_okf_concept_revision',
        'kb_okf_bundle_file', 'kb_okf_schema_profile', 'kb_okf_log_entry',
        'kb_local_mirror_package', 'kb_space_context_binding', 'kb_outbox_event',
        'kb_okf_concept_link', 'kb_okf_candidate', 'kb_market_listing',
        'kb_market_subscription', 'kb_audit_event',
        'kb_group_knowledge_space_binding', 'kb_group_knowledge_space_member',
        'kb_group_knowledge_space_event_inbox', 'kb_group_knowledge_space_membership_projection',
        'kb_provider_credential_reference', 'kb_provider_binding',
        'kb_provider_migration_operation', 'kb_site_publication',
        'kb_source_file_projection', 'kb_source_file_rendition',
        'kb_drive_source_checkpoint', 'kb_drive_event_inbox'
    ])
    LOOP
        EXECUTE format('ALTER TABLE %I ENABLE ROW LEVEL SECURITY', table_name);
        EXECUTE format('ALTER TABLE %I FORCE ROW LEVEL SECURITY', table_name);
        EXECUTE format('DROP POLICY IF EXISTS tenant_isolation ON %I', table_name);
        EXECUTE format('DROP POLICY IF EXISTS organization_isolation ON %I', table_name);
        EXECUTE format(
            'CREATE POLICY organization_isolation ON %I AS PERMISSIVE FOR ALL TO PUBLIC USING (tenant_id = NULLIF(current_setting(''app.current_tenant_id'', true), '''')::bigint AND organization_id = NULLIF(current_setting(''app.current_organization_id'', true), '''')::bigint) WITH CHECK (tenant_id = NULLIF(current_setting(''app.current_tenant_id'', true), '''')::bigint AND organization_id = NULLIF(current_setting(''app.current_organization_id'', true), '''')::bigint)',
            table_name
        );
    END LOOP;
END $$;
