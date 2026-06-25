CREATE TABLE IF NOT EXISTS kb_space (
    id BIGINT NOT NULL,
    uuid TEXT NOT NULL,
    tenant_id INTEGER NOT NULL,
    organization_id INTEGER NOT NULL DEFAULT 0,
    name TEXT NOT NULL,
    description TEXT,
    drive_space_id TEXT,
    status INTEGER NOT NULL,
    okf_bundle_initialized INTEGER NOT NULL DEFAULT 0,
    okf_log_sequence_counter INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    version INTEGER NOT NULL DEFAULT 0,
    PRIMARY KEY (id)
);

CREATE UNIQUE INDEX IF NOT EXISTS uk_kb_space_uuid
    ON kb_space (tenant_id, uuid);

CREATE UNIQUE INDEX IF NOT EXISTS uk_kb_space_drive_space
    ON kb_space (tenant_id, drive_space_id)
    WHERE drive_space_id IS NOT NULL AND status = 1;

CREATE TABLE IF NOT EXISTS kb_collection (
    id BIGINT NOT NULL,
    uuid TEXT NOT NULL,
    tenant_id INTEGER NOT NULL,
    space_id INTEGER NOT NULL,
    parent_id INTEGER NOT NULL DEFAULT 0,
    name TEXT NOT NULL,
    path TEXT NOT NULL,
    level_no INTEGER NOT NULL,
    sort_order INTEGER NOT NULL DEFAULT 0,
    status INTEGER NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    version INTEGER NOT NULL DEFAULT 0,
    PRIMARY KEY (id)
);

CREATE UNIQUE INDEX IF NOT EXISTS uk_kb_collection_uuid
    ON kb_collection (tenant_id, uuid);

CREATE TABLE IF NOT EXISTS kb_source (
    id BIGINT NOT NULL,
    uuid TEXT NOT NULL,
    tenant_id INTEGER NOT NULL,
    space_id INTEGER NOT NULL,
    source_type TEXT NOT NULL,
    provider TEXT,
    drive_bucket TEXT,
    drive_prefix TEXT,
    sync_policy TEXT,
    last_sync_at TEXT,
    last_sync_job_id INTEGER,
    metadata TEXT,
    status INTEGER NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    version INTEGER NOT NULL DEFAULT 0,
    PRIMARY KEY (id)
);

CREATE UNIQUE INDEX IF NOT EXISTS uk_kb_source_uuid
    ON kb_source (tenant_id, uuid);

CREATE UNIQUE INDEX IF NOT EXISTS uk_kb_source_identity
    ON kb_source (
        tenant_id,
        space_id,
        source_type,
        COALESCE(provider, ''),
        COALESCE(drive_bucket, ''),
        COALESCE(drive_prefix, '')
    )
    WHERE status = 1;

CREATE TABLE IF NOT EXISTS kb_drive_object_ref (
    id BIGINT NOT NULL,
    uuid TEXT NOT NULL,
    tenant_id INTEGER NOT NULL,
    space_id INTEGER NOT NULL,
    drive_provider_kind TEXT NOT NULL,
    drive_space_id TEXT,
    drive_node_id TEXT,
    logical_path TEXT,
    drive_storage_provider_id TEXT NOT NULL,
    drive_bucket TEXT NOT NULL,
    drive_object_key TEXT NOT NULL,
    drive_object_version TEXT,
    drive_etag TEXT,
    content_type TEXT,
    size_bytes INTEGER NOT NULL,
    checksum_sha256_hex TEXT,
    drive_metadata TEXT,
    object_role TEXT NOT NULL,
    access_mode TEXT NOT NULL,
    status INTEGER NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    version INTEGER NOT NULL DEFAULT 0,
    PRIMARY KEY (id)
);

CREATE UNIQUE INDEX IF NOT EXISTS uk_kb_drive_object_ref_uuid
    ON kb_drive_object_ref (tenant_id, uuid);

CREATE INDEX IF NOT EXISTS idx_kb_drive_object_locator
    ON kb_drive_object_ref (
        tenant_id,
        drive_storage_provider_id,
        drive_bucket,
        drive_object_key,
        drive_object_version
    );

CREATE UNIQUE INDEX IF NOT EXISTS uk_kb_drive_object_ref_locator
    ON kb_drive_object_ref (
        tenant_id,
        space_id,
        drive_storage_provider_id,
        drive_bucket,
        drive_object_key,
        COALESCE(drive_object_version, ''),
        object_role
    );

CREATE INDEX IF NOT EXISTS idx_kb_drive_object_role
    ON kb_drive_object_ref (tenant_id, object_role, created_at);

CREATE INDEX IF NOT EXISTS idx_kb_drive_object_drive_node
    ON kb_drive_object_ref (tenant_id, space_id, drive_space_id, drive_node_id, status);

CREATE TABLE IF NOT EXISTS kb_document (
    id BIGINT NOT NULL,
    uuid TEXT NOT NULL,
    tenant_id INTEGER NOT NULL,
    space_id INTEGER NOT NULL,
    collection_id INTEGER NOT NULL DEFAULT 0,
    source_id INTEGER,
    identity_scope TEXT NOT NULL DEFAULT 'source_and_original_drive_node',
    original_file_drive_node_id TEXT,
    title TEXT NOT NULL,
    mime_type TEXT,
    language TEXT,
    current_version_id INTEGER,
    visibility INTEGER NOT NULL,
    content_state INTEGER NOT NULL,
    index_state INTEGER NOT NULL,
    metadata TEXT,
    status INTEGER NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    version INTEGER NOT NULL DEFAULT 0,
    PRIMARY KEY (id)
);

CREATE UNIQUE INDEX IF NOT EXISTS uk_kb_document_uuid
    ON kb_document (tenant_id, uuid);

CREATE INDEX IF NOT EXISTS idx_kb_document_drive_node
    ON kb_document (tenant_id, space_id, original_file_drive_node_id, status);

CREATE UNIQUE INDEX IF NOT EXISTS uk_kb_document_identity
    ON kb_document (
        tenant_id,
        space_id,
        collection_id,
        identity_scope,
        COALESCE(source_id, 0),
        CASE
            WHEN identity_scope = 'source_only' THEN ''
            ELSE COALESCE(original_file_drive_node_id, '')
        END
    )
    WHERE status = 1;

CREATE TABLE IF NOT EXISTS kb_document_version (
    id BIGINT NOT NULL,
    uuid TEXT NOT NULL,
    tenant_id INTEGER NOT NULL,
    document_id INTEGER NOT NULL,
    version_no INTEGER NOT NULL,
    original_object_ref_id INTEGER NOT NULL,
    checksum_sha256_hex TEXT,
    size_bytes INTEGER NOT NULL,
    mime_type TEXT,
    parser_profile_id INTEGER,
    parse_state INTEGER NOT NULL,
    index_state INTEGER NOT NULL,
    submitted_by INTEGER,
    submitted_at TEXT NOT NULL,
    metadata TEXT,
    status INTEGER NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    version INTEGER NOT NULL DEFAULT 0,
    PRIMARY KEY (id)
);

CREATE UNIQUE INDEX IF NOT EXISTS uk_kb_document_version_uuid
    ON kb_document_version (tenant_id, uuid);

CREATE UNIQUE INDEX IF NOT EXISTS uk_kb_document_version_no
    ON kb_document_version (tenant_id, document_id, version_no);

CREATE TABLE IF NOT EXISTS kb_chunk (
    id BIGINT NOT NULL,
    uuid TEXT NOT NULL,
    tenant_id INTEGER NOT NULL,
    space_id INTEGER NOT NULL,
    collection_id INTEGER NOT NULL DEFAULT 0,
    document_id INTEGER NOT NULL,
    document_version_id INTEGER NOT NULL,
    chunk_index INTEGER NOT NULL,
    content_text TEXT NOT NULL,
    content_hash TEXT NOT NULL,
    token_count INTEGER,
    locator TEXT,
    metadata TEXT,
    status INTEGER NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    version INTEGER NOT NULL DEFAULT 0,
    PRIMARY KEY (id)
);

CREATE UNIQUE INDEX IF NOT EXISTS uk_kb_chunk_uuid
    ON kb_chunk (tenant_id, uuid);

CREATE UNIQUE INDEX IF NOT EXISTS uk_kb_chunk_document_version_chunk
    ON kb_chunk (tenant_id, document_version_id, chunk_index);

CREATE INDEX IF NOT EXISTS idx_kb_chunk_document_version
    ON kb_chunk (tenant_id, document_version_id, status, chunk_index);

CREATE INDEX IF NOT EXISTS idx_kb_chunk_space_status
    ON kb_chunk (tenant_id, space_id, collection_id, status);

CREATE TABLE IF NOT EXISTS kb_index (
    id BIGINT NOT NULL,
    uuid TEXT NOT NULL,
    tenant_id INTEGER NOT NULL,
    space_id INTEGER NOT NULL,
    collection_id INTEGER NOT NULL DEFAULT 0,
    index_kind TEXT NOT NULL,
    embedding_provider_id TEXT,
    embedding_model TEXT,
    dimension INTEGER,
    metric TEXT,
    schema_version TEXT NOT NULL,
    metadata TEXT,
    status INTEGER NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    version INTEGER NOT NULL DEFAULT 0,
    PRIMARY KEY (id)
);

CREATE UNIQUE INDEX IF NOT EXISTS uk_kb_index_uuid
    ON kb_index (tenant_id, uuid);

CREATE INDEX IF NOT EXISTS idx_kb_index_scope
    ON kb_index (tenant_id, space_id, collection_id, index_kind, status);

CREATE TABLE IF NOT EXISTS kb_embedding (
    id BIGINT NOT NULL,
    uuid TEXT NOT NULL,
    tenant_id INTEGER NOT NULL,
    index_id INTEGER NOT NULL,
    chunk_id INTEGER NOT NULL,
    embedding_hash TEXT NOT NULL,
    vector_ref TEXT NOT NULL,
    dimension INTEGER NOT NULL,
    provider_id TEXT,
    model TEXT,
    metadata TEXT,
    status INTEGER NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    version INTEGER NOT NULL DEFAULT 0,
    PRIMARY KEY (id)
);

CREATE UNIQUE INDEX IF NOT EXISTS uk_kb_embedding_uuid
    ON kb_embedding (tenant_id, uuid);

CREATE UNIQUE INDEX IF NOT EXISTS uk_kb_embedding_index_chunk
    ON kb_embedding (tenant_id, index_id, chunk_id);

CREATE INDEX IF NOT EXISTS idx_kb_embedding_chunk
    ON kb_embedding (tenant_id, chunk_id, status);

CREATE TABLE IF NOT EXISTS kb_retrieval_profile (
    id BIGINT NOT NULL,
    uuid TEXT NOT NULL,
    tenant_id INTEGER NOT NULL,
    name TEXT NOT NULL,
    strategy TEXT NOT NULL,
    top_k INTEGER NOT NULL,
    min_score REAL,
    rerank_enabled INTEGER NOT NULL DEFAULT 0,
    rerank_provider_id TEXT,
    query_rewrite_enabled INTEGER NOT NULL DEFAULT 0,
    citation_policy TEXT,
    filter_policy TEXT,
    context_budget_tokens INTEGER NOT NULL,
    metadata TEXT,
    status INTEGER NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    version INTEGER NOT NULL DEFAULT 0,
    PRIMARY KEY (id)
);

CREATE UNIQUE INDEX IF NOT EXISTS uk_kb_retrieval_profile_uuid
    ON kb_retrieval_profile (tenant_id, uuid);

CREATE INDEX IF NOT EXISTS idx_kb_retrieval_profile_tenant_status
    ON kb_retrieval_profile (tenant_id, status, updated_at);

CREATE TABLE IF NOT EXISTS kb_retrieval_trace (
    id BIGINT NOT NULL,
    uuid TEXT NOT NULL,
    tenant_id INTEGER NOT NULL,
    actor_id INTEGER,
    retrieval_profile_id INTEGER,
    query_hash TEXT NOT NULL,
    query_text_redacted TEXT,
    request_payload TEXT,
    latency_ms INTEGER,
    result_count INTEGER NOT NULL DEFAULT 0,
    error_code TEXT,
    error_detail TEXT,
    status INTEGER NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    version INTEGER NOT NULL DEFAULT 0,
    PRIMARY KEY (id)
);

CREATE UNIQUE INDEX IF NOT EXISTS uk_kb_retrieval_trace_uuid
    ON kb_retrieval_trace (tenant_id, uuid);

CREATE INDEX IF NOT EXISTS idx_kb_retrieval_trace_profile_created
    ON kb_retrieval_trace (tenant_id, retrieval_profile_id, created_at);

CREATE INDEX IF NOT EXISTS idx_kb_retrieval_trace_actor_created
    ON kb_retrieval_trace (tenant_id, actor_id, created_at);

CREATE TABLE IF NOT EXISTS kb_retrieval_hit (
    id BIGINT NOT NULL,
    uuid TEXT NOT NULL,
    tenant_id INTEGER NOT NULL,
    retrieval_trace_id INTEGER NOT NULL,
    chunk_id INTEGER NOT NULL,
    document_id INTEGER NOT NULL,
    document_version_id INTEGER,
    score REAL,
    result_rank INTEGER NOT NULL,
    match_reason TEXT,
    citation TEXT,
    metadata TEXT,
    status INTEGER NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    version INTEGER NOT NULL DEFAULT 0,
    PRIMARY KEY (id)
);

CREATE UNIQUE INDEX IF NOT EXISTS uk_kb_retrieval_hit_uuid
    ON kb_retrieval_hit (tenant_id, uuid);

CREATE INDEX IF NOT EXISTS idx_kb_retrieval_hit_trace_rank
    ON kb_retrieval_hit (tenant_id, retrieval_trace_id, result_rank);

CREATE INDEX IF NOT EXISTS idx_kb_retrieval_hit_chunk
    ON kb_retrieval_hit (tenant_id, chunk_id, status);

CREATE TABLE IF NOT EXISTS kb_agent_profile (
    id BIGINT NOT NULL,
    uuid TEXT NOT NULL,
    tenant_id INTEGER NOT NULL,
    name TEXT NOT NULL,
    description TEXT,
    system_instruction TEXT NOT NULL,
    model_provider_id TEXT NOT NULL,
    model_id TEXT NOT NULL,
    model_parameters TEXT,
    retrieval_profile_id INTEGER,
    citation_policy TEXT,
    memory_policy_ref TEXT,
    tool_policy_ref TEXT,
    answer_policy TEXT,
    metadata TEXT,
    status INTEGER NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    version INTEGER NOT NULL DEFAULT 0,
    PRIMARY KEY (id)
);

CREATE UNIQUE INDEX IF NOT EXISTS uk_kb_agent_profile_uuid
    ON kb_agent_profile (tenant_id, uuid);

CREATE INDEX IF NOT EXISTS idx_kb_agent_profile_model
    ON kb_agent_profile (tenant_id, model_provider_id, model_id, status);

CREATE TABLE IF NOT EXISTS kb_agent_knowledge_binding (
    id BIGINT NOT NULL,
    uuid TEXT NOT NULL,
    tenant_id INTEGER NOT NULL,
    profile_id INTEGER NOT NULL,
    space_id INTEGER NOT NULL,
    collection_id INTEGER,
    source_filter TEXT,
    document_filter TEXT,
    priority INTEGER NOT NULL DEFAULT 0,
    top_k INTEGER,
    min_score REAL,
    enabled INTEGER NOT NULL DEFAULT 1,
    status INTEGER NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    version INTEGER NOT NULL DEFAULT 0,
    PRIMARY KEY (id)
);

CREATE UNIQUE INDEX IF NOT EXISTS uk_kb_agent_knowledge_binding_uuid
    ON kb_agent_knowledge_binding (tenant_id, uuid);

CREATE INDEX IF NOT EXISTS idx_kb_agent_knowledge_binding_profile
    ON kb_agent_knowledge_binding (tenant_id, profile_id, enabled, priority);

CREATE TABLE IF NOT EXISTS kb_ingestion_job (
    id BIGINT NOT NULL,
    uuid TEXT NOT NULL,
    tenant_id INTEGER NOT NULL,
    space_id INTEGER NOT NULL,
    source_id INTEGER,
    job_type TEXT NOT NULL,
    state INTEGER NOT NULL,
    priority INTEGER NOT NULL DEFAULT 0,
    progress INTEGER NOT NULL DEFAULT 0,
    requested_by INTEGER,
    idempotency_key TEXT NOT NULL,
    request_id TEXT,
    trace_id TEXT,
    error_code TEXT,
    error_detail TEXT,
    started_at TEXT,
    finished_at TEXT,
    metadata TEXT,
    status INTEGER NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    version INTEGER NOT NULL DEFAULT 0,
    PRIMARY KEY (id)
);

CREATE UNIQUE INDEX IF NOT EXISTS uk_kb_ingestion_job_uuid
    ON kb_ingestion_job (tenant_id, uuid);

CREATE UNIQUE INDEX IF NOT EXISTS uk_kb_ingestion_job_idempotency
    ON kb_ingestion_job (tenant_id, space_id, idempotency_key);

CREATE TABLE IF NOT EXISTS kb_ingestion_job_item (
    id BIGINT NOT NULL,
    uuid TEXT NOT NULL,
    tenant_id INTEGER NOT NULL,
    job_id INTEGER NOT NULL,
    document_id INTEGER,
    document_version_id INTEGER,
    input_object_ref_id INTEGER,
    stage TEXT NOT NULL,
    state INTEGER NOT NULL,
    attempt_count INTEGER NOT NULL DEFAULT 0,
    error_code TEXT,
    error_detail TEXT,
    started_at TEXT,
    finished_at TEXT,
    status INTEGER NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    version INTEGER NOT NULL DEFAULT 0,
    PRIMARY KEY (id)
);

CREATE UNIQUE INDEX IF NOT EXISTS uk_kb_ingestion_job_item_uuid
    ON kb_ingestion_job_item (tenant_id, uuid);

CREATE TABLE IF NOT EXISTS kb_okf_concept (
    id BIGINT NOT NULL,
    uuid TEXT NOT NULL,
    tenant_id INTEGER NOT NULL,
    space_id INTEGER NOT NULL,
    concept_id TEXT NOT NULL,
    title TEXT NOT NULL,
    concept_type TEXT NOT NULL,
    logical_path TEXT NOT NULL,
    description TEXT NOT NULL DEFAULT '',
    source_count INTEGER NOT NULL DEFAULT 0,
    tags TEXT,
    current_revision_id INTEGER,
    publish_state TEXT NOT NULL,
    revision_counter INTEGER NOT NULL DEFAULT 0,
    status INTEGER NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    version INTEGER NOT NULL DEFAULT 0,
    PRIMARY KEY (id)
);

CREATE UNIQUE INDEX IF NOT EXISTS uk_kb_okf_concept_uuid
    ON kb_okf_concept (tenant_id, uuid);

CREATE UNIQUE INDEX IF NOT EXISTS uk_kb_okf_concept_id
    ON kb_okf_concept (tenant_id, space_id, concept_id);

CREATE UNIQUE INDEX IF NOT EXISTS uk_kb_okf_concept_path
    ON kb_okf_concept (tenant_id, space_id, logical_path);

CREATE INDEX IF NOT EXISTS idx_kb_okf_concept_state
    ON kb_okf_concept (tenant_id, space_id, publish_state, updated_at);

CREATE TABLE IF NOT EXISTS kb_okf_concept_revision (
    id BIGINT NOT NULL,
    uuid TEXT NOT NULL,
    tenant_id INTEGER NOT NULL,
    concept_row_id INTEGER NOT NULL,
    revision_no INTEGER NOT NULL,
    markdown_object_ref_id INTEGER NOT NULL,
    content_hash TEXT NOT NULL,
    review_state TEXT NOT NULL,
    status INTEGER NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    version INTEGER NOT NULL DEFAULT 0,
    PRIMARY KEY (id)
);

CREATE UNIQUE INDEX IF NOT EXISTS uk_kb_okf_concept_revision_uuid
    ON kb_okf_concept_revision (tenant_id, uuid);

CREATE UNIQUE INDEX IF NOT EXISTS uk_kb_okf_concept_revision_no
    ON kb_okf_concept_revision (tenant_id, concept_row_id, revision_no);

CREATE TABLE IF NOT EXISTS kb_okf_bundle_file (
    id BIGINT NOT NULL,
    uuid TEXT NOT NULL,
    tenant_id INTEGER NOT NULL,
    space_id INTEGER NOT NULL,
    logical_path TEXT NOT NULL,
    file_kind TEXT NOT NULL,
    artifact_role TEXT NOT NULL,
    drive_bucket TEXT NOT NULL,
    drive_object_key TEXT NOT NULL,
    checksum_sha256_hex TEXT,
    status INTEGER NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    version INTEGER NOT NULL DEFAULT 0,
    PRIMARY KEY (id)
);

CREATE UNIQUE INDEX IF NOT EXISTS uk_kb_okf_bundle_file_uuid
    ON kb_okf_bundle_file (tenant_id, uuid);

CREATE UNIQUE INDEX IF NOT EXISTS uk_kb_okf_bundle_file_path
    ON kb_okf_bundle_file (tenant_id, space_id, logical_path);

CREATE TABLE IF NOT EXISTS kb_okf_schema_profile (
    id BIGINT NOT NULL,
    uuid TEXT NOT NULL,
    tenant_id INTEGER NOT NULL,
    space_id INTEGER NOT NULL,
    profile_version TEXT NOT NULL,
    schema_object_ref_id INTEGER NOT NULL,
    agents_md_object_ref_id INTEGER NOT NULL,
    state TEXT NOT NULL,
    status INTEGER NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    version INTEGER NOT NULL DEFAULT 0,
    PRIMARY KEY (id)
);

CREATE UNIQUE INDEX IF NOT EXISTS uk_kb_okf_schema_profile_uuid
    ON kb_okf_schema_profile (tenant_id, uuid);

CREATE TABLE IF NOT EXISTS kb_okf_log_entry (
    id BIGINT NOT NULL,
    uuid TEXT NOT NULL,
    tenant_id INTEGER NOT NULL,
    space_id INTEGER NOT NULL,
    sequence_no INTEGER NOT NULL,
    event_type TEXT NOT NULL,
    event_time TEXT NOT NULL,
    title TEXT NOT NULL,
    privacy_level TEXT NOT NULL,
    metadata TEXT,
    status INTEGER NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    version INTEGER NOT NULL DEFAULT 0,
    PRIMARY KEY (id)
);

CREATE UNIQUE INDEX IF NOT EXISTS uk_kb_okf_log_entry_uuid
    ON kb_okf_log_entry (tenant_id, uuid);

CREATE UNIQUE INDEX IF NOT EXISTS uk_kb_okf_log_entry_sequence
    ON kb_okf_log_entry (tenant_id, space_id, sequence_no);

CREATE TABLE IF NOT EXISTS kb_local_mirror_package (
    id BIGINT NOT NULL,
    uuid TEXT NOT NULL,
    tenant_id INTEGER NOT NULL,
    space_id INTEGER NOT NULL,
    package_type TEXT NOT NULL,
    snapshot_version TEXT NOT NULL,
    object_ref_id INTEGER NOT NULL,
    manifest_object_ref_id INTEGER NOT NULL,
    checksum_sha256_hex TEXT NOT NULL,
    state TEXT NOT NULL,
    status INTEGER NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    version INTEGER NOT NULL DEFAULT 0,
    PRIMARY KEY (id)
);

CREATE UNIQUE INDEX IF NOT EXISTS uk_kb_local_mirror_package_uuid
    ON kb_local_mirror_package (tenant_id, uuid);
