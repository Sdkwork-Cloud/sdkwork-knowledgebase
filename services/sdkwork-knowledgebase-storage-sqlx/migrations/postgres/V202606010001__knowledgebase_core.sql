CREATE TABLE IF NOT EXISTS kb_space (
    id BIGINT PRIMARY KEY,
    uuid VARCHAR(64) NOT NULL,
    tenant_id BIGINT NOT NULL,
    organization_id BIGINT NOT NULL DEFAULT 0,
    name VARCHAR(200) NOT NULL,
    description TEXT,
    drive_space_id VARCHAR(128),
    status INTEGER NOT NULL,
    llm_wiki_initialized BOOLEAN NOT NULL DEFAULT FALSE,
    wiki_log_sequence_counter BIGINT NOT NULL DEFAULT 0,
    created_at TIMESTAMP NOT NULL,
    updated_at TIMESTAMP NOT NULL,
    version BIGINT NOT NULL DEFAULT 0
);

CREATE UNIQUE INDEX IF NOT EXISTS uk_kb_space_uuid
    ON kb_space (tenant_id, uuid);

CREATE UNIQUE INDEX IF NOT EXISTS uk_kb_space_drive_space
    ON kb_space (tenant_id, drive_space_id)
    WHERE drive_space_id IS NOT NULL AND status = 1;

CREATE TABLE IF NOT EXISTS kb_collection (
    id BIGINT PRIMARY KEY,
    uuid VARCHAR(64) NOT NULL,
    tenant_id BIGINT NOT NULL,
    space_id BIGINT NOT NULL,
    parent_id BIGINT NOT NULL DEFAULT 0,
    name VARCHAR(200) NOT NULL,
    path VARCHAR(2048) NOT NULL,
    level_no INTEGER NOT NULL,
    sort_order INTEGER NOT NULL DEFAULT 0,
    status INTEGER NOT NULL,
    created_at TIMESTAMP NOT NULL,
    updated_at TIMESTAMP NOT NULL,
    version BIGINT NOT NULL DEFAULT 0
);

CREATE UNIQUE INDEX IF NOT EXISTS uk_kb_collection_uuid
    ON kb_collection (tenant_id, uuid);

CREATE TABLE IF NOT EXISTS kb_source (
    id BIGINT PRIMARY KEY,
    uuid VARCHAR(64) NOT NULL,
    tenant_id BIGINT NOT NULL,
    space_id BIGINT NOT NULL,
    source_type VARCHAR(64) NOT NULL,
    provider VARCHAR(128),
    drive_bucket VARCHAR(256),
    drive_prefix VARCHAR(1024),
    sync_policy JSONB,
    last_sync_at TIMESTAMP,
    last_sync_job_id BIGINT,
    metadata JSONB,
    status INTEGER NOT NULL,
    created_at TIMESTAMP NOT NULL,
    updated_at TIMESTAMP NOT NULL,
    version BIGINT NOT NULL DEFAULT 0
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
    id BIGINT PRIMARY KEY,
    uuid VARCHAR(64) NOT NULL,
    tenant_id BIGINT NOT NULL,
    space_id BIGINT NOT NULL,
    drive_provider_kind VARCHAR(64) NOT NULL,
    drive_space_id VARCHAR(128),
    drive_node_id VARCHAR(128),
    logical_path TEXT,
    drive_bucket VARCHAR(256) NOT NULL,
    drive_object_key VARCHAR(2048) NOT NULL,
    drive_object_version VARCHAR(256),
    drive_etag VARCHAR(256),
    content_type VARCHAR(256),
    size_bytes BIGINT NOT NULL,
    checksum_sha256_hex VARCHAR(128),
    drive_metadata JSONB,
    object_role VARCHAR(64) NOT NULL,
    access_mode VARCHAR(64) NOT NULL,
    status INTEGER NOT NULL,
    created_at TIMESTAMP NOT NULL,
    updated_at TIMESTAMP NOT NULL,
    version BIGINT NOT NULL DEFAULT 0
);

CREATE UNIQUE INDEX IF NOT EXISTS uk_kb_drive_object_ref_uuid
    ON kb_drive_object_ref (tenant_id, uuid);

CREATE INDEX IF NOT EXISTS idx_kb_drive_object_locator
    ON kb_drive_object_ref (tenant_id, drive_bucket, drive_object_key, drive_object_version);

CREATE UNIQUE INDEX IF NOT EXISTS uk_kb_drive_object_ref_locator
    ON kb_drive_object_ref (
        tenant_id,
        space_id,
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
    id BIGINT PRIMARY KEY,
    uuid VARCHAR(64) NOT NULL,
    tenant_id BIGINT NOT NULL,
    space_id BIGINT NOT NULL,
    collection_id BIGINT NOT NULL DEFAULT 0,
    source_id BIGINT,
    identity_scope VARCHAR(64) NOT NULL DEFAULT 'source_and_original_drive_node',
    original_file_drive_node_id VARCHAR(128),
    title VARCHAR(512) NOT NULL,
    mime_type VARCHAR(256),
    language VARCHAR(32),
    current_version_id BIGINT,
    visibility INTEGER NOT NULL,
    content_state INTEGER NOT NULL,
    index_state INTEGER NOT NULL,
    metadata JSONB,
    status INTEGER NOT NULL,
    created_at TIMESTAMP NOT NULL,
    updated_at TIMESTAMP NOT NULL,
    version BIGINT NOT NULL DEFAULT 0
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
    id BIGINT PRIMARY KEY,
    uuid VARCHAR(64) NOT NULL,
    tenant_id BIGINT NOT NULL,
    document_id BIGINT NOT NULL,
    version_no BIGINT NOT NULL,
    original_object_ref_id BIGINT NOT NULL,
    checksum_sha256_hex VARCHAR(128),
    size_bytes BIGINT NOT NULL,
    mime_type VARCHAR(256),
    parser_profile_id BIGINT,
    parse_state INTEGER NOT NULL,
    index_state INTEGER NOT NULL,
    submitted_by BIGINT,
    submitted_at TIMESTAMP NOT NULL,
    metadata JSONB,
    status INTEGER NOT NULL,
    created_at TIMESTAMP NOT NULL,
    updated_at TIMESTAMP NOT NULL,
    version BIGINT NOT NULL DEFAULT 0
);

CREATE UNIQUE INDEX IF NOT EXISTS uk_kb_document_version_uuid
    ON kb_document_version (tenant_id, uuid);

CREATE UNIQUE INDEX IF NOT EXISTS uk_kb_document_version_no
    ON kb_document_version (tenant_id, document_id, version_no);

CREATE TABLE IF NOT EXISTS kb_ingestion_job (
    id BIGINT PRIMARY KEY,
    uuid VARCHAR(64) NOT NULL,
    tenant_id BIGINT NOT NULL,
    space_id BIGINT NOT NULL,
    source_id BIGINT,
    job_type VARCHAR(64) NOT NULL,
    state INTEGER NOT NULL,
    priority INTEGER NOT NULL DEFAULT 0,
    progress INTEGER NOT NULL DEFAULT 0,
    requested_by BIGINT,
    idempotency_key VARCHAR(128) NOT NULL,
    request_id VARCHAR(64),
    trace_id VARCHAR(128),
    error_code VARCHAR(128),
    error_detail VARCHAR(4000),
    started_at TIMESTAMP,
    finished_at TIMESTAMP,
    metadata JSONB,
    status INTEGER NOT NULL,
    created_at TIMESTAMP NOT NULL,
    updated_at TIMESTAMP NOT NULL,
    version BIGINT NOT NULL DEFAULT 0
);

CREATE UNIQUE INDEX IF NOT EXISTS uk_kb_ingestion_job_uuid
    ON kb_ingestion_job (tenant_id, uuid);

CREATE UNIQUE INDEX IF NOT EXISTS uk_kb_ingestion_job_idempotency
    ON kb_ingestion_job (tenant_id, space_id, idempotency_key);

CREATE TABLE IF NOT EXISTS kb_ingestion_job_item (
    id BIGINT PRIMARY KEY,
    uuid VARCHAR(64) NOT NULL,
    tenant_id BIGINT NOT NULL,
    job_id BIGINT NOT NULL,
    document_id BIGINT,
    document_version_id BIGINT,
    input_object_ref_id BIGINT,
    stage VARCHAR(64) NOT NULL,
    state INTEGER NOT NULL,
    attempt_count INTEGER NOT NULL DEFAULT 0,
    error_code VARCHAR(128),
    error_detail VARCHAR(4000),
    started_at TIMESTAMP,
    finished_at TIMESTAMP,
    status INTEGER NOT NULL,
    created_at TIMESTAMP NOT NULL,
    updated_at TIMESTAMP NOT NULL,
    version BIGINT NOT NULL DEFAULT 0
);

CREATE UNIQUE INDEX IF NOT EXISTS uk_kb_ingestion_job_item_uuid
    ON kb_ingestion_job_item (tenant_id, uuid);

CREATE TABLE IF NOT EXISTS kb_wiki_page (
    id BIGINT PRIMARY KEY,
    uuid VARCHAR(64) NOT NULL,
    tenant_id BIGINT NOT NULL,
    space_id BIGINT NOT NULL,
    slug VARCHAR(256) NOT NULL,
    title VARCHAR(512) NOT NULL,
    page_type VARCHAR(64) NOT NULL,
    logical_path VARCHAR(2048) NOT NULL,
    summary TEXT NOT NULL DEFAULT '',
    source_count INTEGER NOT NULL DEFAULT 0,
    tags JSONB,
    current_revision_id BIGINT,
    publish_state VARCHAR(64) NOT NULL,
    revision_counter BIGINT NOT NULL DEFAULT 0,
    status INTEGER NOT NULL,
    created_at TIMESTAMP NOT NULL,
    updated_at TIMESTAMP NOT NULL,
    version BIGINT NOT NULL DEFAULT 0
);

CREATE UNIQUE INDEX IF NOT EXISTS uk_kb_wiki_page_uuid
    ON kb_wiki_page (tenant_id, uuid);

CREATE UNIQUE INDEX IF NOT EXISTS uk_kb_wiki_page_slug
    ON kb_wiki_page (tenant_id, space_id, slug);

CREATE UNIQUE INDEX IF NOT EXISTS uk_kb_wiki_page_path
    ON kb_wiki_page (tenant_id, space_id, logical_path);

CREATE INDEX IF NOT EXISTS idx_kb_wiki_page_state
    ON kb_wiki_page (tenant_id, space_id, publish_state, updated_at);

CREATE TABLE IF NOT EXISTS kb_wiki_page_revision (
    id BIGINT PRIMARY KEY,
    uuid VARCHAR(64) NOT NULL,
    tenant_id BIGINT NOT NULL,
    page_id BIGINT NOT NULL,
    revision_no BIGINT NOT NULL,
    markdown_object_ref_id BIGINT NOT NULL,
    content_hash VARCHAR(128) NOT NULL,
    review_state VARCHAR(64) NOT NULL,
    status INTEGER NOT NULL,
    created_at TIMESTAMP NOT NULL,
    updated_at TIMESTAMP NOT NULL,
    version BIGINT NOT NULL DEFAULT 0
);

CREATE UNIQUE INDEX IF NOT EXISTS uk_kb_wiki_page_revision_uuid
    ON kb_wiki_page_revision (tenant_id, uuid);

CREATE UNIQUE INDEX IF NOT EXISTS uk_kb_wiki_page_revision_no
    ON kb_wiki_page_revision (tenant_id, page_id, revision_no);

CREATE TABLE IF NOT EXISTS kb_wiki_file_entry (
    id BIGINT PRIMARY KEY,
    uuid VARCHAR(64) NOT NULL,
    tenant_id BIGINT NOT NULL,
    space_id BIGINT NOT NULL,
    logical_path VARCHAR(2048) NOT NULL,
    entry_type VARCHAR(64) NOT NULL,
    artifact_role VARCHAR(64) NOT NULL,
    drive_bucket VARCHAR(256) NOT NULL,
    drive_object_key VARCHAR(2048) NOT NULL,
    checksum_sha256_hex VARCHAR(128),
    status INTEGER NOT NULL,
    created_at TIMESTAMP NOT NULL,
    updated_at TIMESTAMP NOT NULL,
    version BIGINT NOT NULL DEFAULT 0
);

CREATE UNIQUE INDEX IF NOT EXISTS uk_kb_wiki_file_entry_uuid
    ON kb_wiki_file_entry (tenant_id, uuid);

CREATE UNIQUE INDEX IF NOT EXISTS uk_kb_wiki_file_entry_path
    ON kb_wiki_file_entry (tenant_id, space_id, logical_path);

CREATE TABLE IF NOT EXISTS kb_wiki_schema_profile (
    id BIGINT PRIMARY KEY,
    uuid VARCHAR(64) NOT NULL,
    tenant_id BIGINT NOT NULL,
    space_id BIGINT NOT NULL,
    profile_version VARCHAR(128) NOT NULL,
    schema_object_ref_id BIGINT NOT NULL,
    agents_md_object_ref_id BIGINT NOT NULL,
    state VARCHAR(64) NOT NULL,
    status INTEGER NOT NULL,
    created_at TIMESTAMP NOT NULL,
    updated_at TIMESTAMP NOT NULL,
    version BIGINT NOT NULL DEFAULT 0
);

CREATE UNIQUE INDEX IF NOT EXISTS uk_kb_wiki_schema_profile_uuid
    ON kb_wiki_schema_profile (tenant_id, uuid);

CREATE TABLE IF NOT EXISTS kb_wiki_log_entry (
    id BIGINT PRIMARY KEY,
    uuid VARCHAR(64) NOT NULL,
    tenant_id BIGINT NOT NULL,
    space_id BIGINT NOT NULL,
    sequence_no BIGINT NOT NULL,
    event_type VARCHAR(64) NOT NULL,
    event_time TIMESTAMP NOT NULL,
    title VARCHAR(512) NOT NULL,
    privacy_level VARCHAR(64) NOT NULL,
    metadata JSONB,
    status INTEGER NOT NULL,
    created_at TIMESTAMP NOT NULL,
    updated_at TIMESTAMP NOT NULL,
    version BIGINT NOT NULL DEFAULT 0
);

CREATE UNIQUE INDEX IF NOT EXISTS uk_kb_wiki_log_entry_uuid
    ON kb_wiki_log_entry (tenant_id, uuid);

CREATE UNIQUE INDEX IF NOT EXISTS uk_kb_wiki_log_entry_sequence
    ON kb_wiki_log_entry (tenant_id, space_id, sequence_no);

CREATE TABLE IF NOT EXISTS kb_local_mirror_package (
    id BIGINT PRIMARY KEY,
    uuid VARCHAR(64) NOT NULL,
    tenant_id BIGINT NOT NULL,
    space_id BIGINT NOT NULL,
    package_type VARCHAR(64) NOT NULL,
    snapshot_version VARCHAR(128) NOT NULL,
    object_ref_id BIGINT NOT NULL,
    manifest_object_ref_id BIGINT NOT NULL,
    checksum_sha256_hex VARCHAR(128) NOT NULL,
    state VARCHAR(64) NOT NULL,
    status INTEGER NOT NULL,
    created_at TIMESTAMP NOT NULL,
    updated_at TIMESTAMP NOT NULL,
    version BIGINT NOT NULL DEFAULT 0
);

CREATE UNIQUE INDEX IF NOT EXISTS uk_kb_local_mirror_package_uuid
    ON kb_local_mirror_package (tenant_id, uuid);
