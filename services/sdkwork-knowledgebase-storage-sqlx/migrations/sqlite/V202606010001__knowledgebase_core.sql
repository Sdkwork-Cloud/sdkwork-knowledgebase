CREATE TABLE IF NOT EXISTS knowledge_space (
    id INTEGER PRIMARY KEY,
    uuid TEXT NOT NULL,
    tenant_id INTEGER NOT NULL,
    organization_id INTEGER NOT NULL DEFAULT 0,
    name TEXT NOT NULL,
    description TEXT,
    status INTEGER NOT NULL,
    llm_wiki_initialized INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    version INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS knowledge_collection (
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

CREATE TABLE IF NOT EXISTS knowledge_source (
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

CREATE TABLE IF NOT EXISTS knowledge_drive_object_ref (
    id INTEGER PRIMARY KEY,
    uuid TEXT NOT NULL,
    tenant_id INTEGER NOT NULL,
    space_id INTEGER NOT NULL,
    drive_provider_kind TEXT NOT NULL,
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

CREATE INDEX IF NOT EXISTS idx_knowledge_drive_object_locator
    ON knowledge_drive_object_ref (tenant_id, drive_bucket, drive_object_key, drive_object_version);

CREATE INDEX IF NOT EXISTS idx_knowledge_drive_object_role
    ON knowledge_drive_object_ref (tenant_id, object_role, created_at);

CREATE TABLE IF NOT EXISTS knowledge_document (
    id INTEGER PRIMARY KEY,
    uuid TEXT NOT NULL,
    tenant_id INTEGER NOT NULL,
    space_id INTEGER NOT NULL,
    collection_id INTEGER NOT NULL DEFAULT 0,
    source_id INTEGER,
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

CREATE TABLE IF NOT EXISTS knowledge_document_version (
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

CREATE TABLE IF NOT EXISTS knowledge_ingestion_job (
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
    idempotency_key TEXT,
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

CREATE TABLE IF NOT EXISTS knowledge_ingestion_job_item (
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

CREATE TABLE IF NOT EXISTS knowledge_wiki_page (
    id INTEGER PRIMARY KEY,
    uuid TEXT NOT NULL,
    tenant_id INTEGER NOT NULL,
    space_id INTEGER NOT NULL,
    slug TEXT NOT NULL,
    title TEXT NOT NULL,
    page_type TEXT NOT NULL,
    current_revision_id INTEGER,
    publish_state TEXT NOT NULL,
    status INTEGER NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    version INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS knowledge_wiki_page_revision (
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

CREATE TABLE IF NOT EXISTS knowledge_wiki_file_entry (
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

CREATE TABLE IF NOT EXISTS knowledge_wiki_schema_profile (
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

CREATE TABLE IF NOT EXISTS knowledge_wiki_log_entry (
    id INTEGER PRIMARY KEY,
    uuid TEXT NOT NULL,
    tenant_id INTEGER NOT NULL,
    space_id INTEGER NOT NULL,
    sequence_no INTEGER NOT NULL,
    event_type TEXT NOT NULL,
    event_time TEXT NOT NULL,
    title TEXT NOT NULL,
    privacy_level TEXT NOT NULL,
    status INTEGER NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    version INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS knowledge_local_mirror_package (
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
