CREATE TABLE IF NOT EXISTS kb_space (
    id INTEGER PRIMARY KEY,
    uuid TEXT NOT NULL,
    tenant_id INTEGER NOT NULL,
    organization_id INTEGER NOT NULL DEFAULT 0,
    name TEXT NOT NULL,
    description TEXT,
    drive_space_id TEXT,
    status INTEGER NOT NULL,
    llm_wiki_initialized INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    version INTEGER NOT NULL DEFAULT 0
);

CREATE UNIQUE INDEX IF NOT EXISTS uk_kb_space_uuid
    ON kb_space (tenant_id, uuid);

CREATE UNIQUE INDEX IF NOT EXISTS uk_kb_space_drive_space
    ON kb_space (tenant_id, drive_space_id)
    WHERE drive_space_id IS NOT NULL;

CREATE TABLE IF NOT EXISTS kb_collection (
    id INTEGER PRIMARY KEY,
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
    version INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS kb_source (
    id INTEGER PRIMARY KEY,
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
    version INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS kb_drive_object_ref (
    id INTEGER PRIMARY KEY,
    uuid TEXT NOT NULL,
    tenant_id INTEGER NOT NULL,
    space_id INTEGER NOT NULL,
    drive_provider_kind TEXT NOT NULL,
    drive_space_id TEXT,
    drive_node_id TEXT,
    logical_path TEXT,
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
    version INTEGER NOT NULL DEFAULT 0
);

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
    id INTEGER PRIMARY KEY,
    uuid TEXT NOT NULL,
    tenant_id INTEGER NOT NULL,
    space_id INTEGER NOT NULL,
    collection_id INTEGER NOT NULL DEFAULT 0,
    source_id INTEGER,
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
    version INTEGER NOT NULL DEFAULT 0
);

CREATE INDEX IF NOT EXISTS idx_kb_document_drive_node
    ON kb_document (tenant_id, space_id, original_file_drive_node_id, status);

CREATE TABLE IF NOT EXISTS kb_document_version (
    id INTEGER PRIMARY KEY,
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
    version INTEGER NOT NULL DEFAULT 0
);

CREATE UNIQUE INDEX IF NOT EXISTS uk_kb_document_version_no
    ON kb_document_version (tenant_id, document_id, version_no);

CREATE TABLE IF NOT EXISTS kb_ingestion_job (
    id INTEGER PRIMARY KEY,
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
    version INTEGER NOT NULL DEFAULT 0
);

CREATE UNIQUE INDEX IF NOT EXISTS uk_kb_ingestion_job_idempotency
    ON kb_ingestion_job (tenant_id, space_id, idempotency_key);

CREATE TABLE IF NOT EXISTS kb_ingestion_job_item (
    id INTEGER PRIMARY KEY,
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
    version INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS kb_wiki_page (
    id INTEGER PRIMARY KEY,
    uuid TEXT NOT NULL,
    tenant_id INTEGER NOT NULL,
    space_id INTEGER NOT NULL,
    slug TEXT NOT NULL,
    title TEXT NOT NULL,
    page_type TEXT NOT NULL,
    logical_path TEXT NOT NULL,
    summary TEXT NOT NULL DEFAULT '',
    source_count INTEGER NOT NULL DEFAULT 0,
    tags TEXT,
    current_revision_id INTEGER,
    publish_state TEXT NOT NULL,
    status INTEGER NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    version INTEGER NOT NULL DEFAULT 0
);

CREATE UNIQUE INDEX IF NOT EXISTS uk_kb_wiki_page_slug
    ON kb_wiki_page (tenant_id, space_id, slug);

CREATE UNIQUE INDEX IF NOT EXISTS uk_kb_wiki_page_path
    ON kb_wiki_page (tenant_id, space_id, logical_path);

CREATE INDEX IF NOT EXISTS idx_kb_wiki_page_state
    ON kb_wiki_page (tenant_id, space_id, publish_state, updated_at);

CREATE TABLE IF NOT EXISTS kb_wiki_page_revision (
    id INTEGER PRIMARY KEY,
    uuid TEXT NOT NULL,
    tenant_id INTEGER NOT NULL,
    page_id INTEGER NOT NULL,
    revision_no INTEGER NOT NULL,
    markdown_object_ref_id INTEGER NOT NULL,
    content_hash TEXT NOT NULL,
    review_state TEXT NOT NULL,
    status INTEGER NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    version INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS kb_wiki_file_entry (
    id INTEGER PRIMARY KEY,
    uuid TEXT NOT NULL,
    tenant_id INTEGER NOT NULL,
    space_id INTEGER NOT NULL,
    logical_path TEXT NOT NULL,
    entry_type TEXT NOT NULL,
    artifact_role TEXT NOT NULL,
    drive_bucket TEXT NOT NULL,
    drive_object_key TEXT NOT NULL,
    checksum_sha256_hex TEXT,
    status INTEGER NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    version INTEGER NOT NULL DEFAULT 0
);

CREATE UNIQUE INDEX IF NOT EXISTS uk_kb_wiki_file_entry_path
    ON kb_wiki_file_entry (tenant_id, space_id, logical_path);

CREATE TABLE IF NOT EXISTS kb_wiki_schema_profile (
    id INTEGER PRIMARY KEY,
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
    version INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS kb_wiki_log_entry (
    id INTEGER PRIMARY KEY,
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
    version INTEGER NOT NULL DEFAULT 0
);

CREATE UNIQUE INDEX IF NOT EXISTS uk_kb_wiki_log_entry_sequence
    ON kb_wiki_log_entry (tenant_id, space_id, sequence_no);

CREATE TABLE IF NOT EXISTS kb_local_mirror_package (
    id INTEGER PRIMARY KEY,
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
    version INTEGER NOT NULL DEFAULT 0
);
