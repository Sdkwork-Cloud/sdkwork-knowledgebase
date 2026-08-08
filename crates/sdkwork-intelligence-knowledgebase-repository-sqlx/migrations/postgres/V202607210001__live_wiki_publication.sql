-- sdkwork:migration
-- id: 202607210001_live_wiki_publication
-- engine: postgres
-- module: knowledgebase
-- purpose: Add canonical live Wiki publication, projection, rendition, checkpoint, and inbox state
-- reversible: true
-- transactional: true
-- lock: lightweight
-- contract_version: 1.1.0

-- Composite Wiki foreign keys require this shared parent key before child tables exist.
CREATE UNIQUE INDEX IF NOT EXISTS uk_kb_space_provider_scope
    ON kb_space (tenant_id, organization_id, id);

CREATE TABLE IF NOT EXISTS kb_site_publication (
    id BIGINT NOT NULL PRIMARY KEY,
    uuid VARCHAR(64) NOT NULL,
    tenant_id BIGINT NOT NULL,
    organization_id BIGINT NOT NULL,
    space_id BIGINT NOT NULL,
    drive_space_uuid VARCHAR(64) NOT NULL,
    source_root_node_uuid VARCHAR(64),
    source_scope_uuid VARCHAR(64),
    publication_type VARCHAR(32) NOT NULL DEFAULT 'wiki',
    wiki_status VARCHAR(32) NOT NULL DEFAULT 'DRAFT',
    title VARCHAR(256) NOT NULL,
    description VARCHAR(2048),
    homepage_source_path VARCHAR(1024) NOT NULL DEFAULT 'index.md',
    default_locale VARCHAR(35) NOT NULL DEFAULT 'zh-CN',
    supported_locales_json JSONB NOT NULL DEFAULT '["zh-CN"]'::jsonb,
    publication_mode VARCHAR(32) NOT NULL DEFAULT 'REVIEW_REQUIRED',
    default_visibility VARCHAR(16) NOT NULL DEFAULT 'PRIVATE',
    update_policy VARCHAR(64) NOT NULL DEFAULT 'KEEP_LAST_PUBLIC_UNTIL_READY',
    navigation_mode VARCHAR(32) NOT NULL DEFAULT 'DIRECTORY',
    navigation_config_json JSONB NOT NULL DEFAULT '{}'::jsonb,
    theme_key VARCHAR(128) NOT NULL DEFAULT 'sdkwork-wiki-default',
    theme_version VARCHAR(64) NOT NULL DEFAULT '1',
    theme_config_json JSONB NOT NULL DEFAULT '{}'::jsonb,
    renderer_policy_version VARCHAR(64) NOT NULL DEFAULT '1',
    search_enabled BOOLEAN NOT NULL DEFAULT TRUE,
    robots_policy VARCHAR(32) NOT NULL DEFAULT 'NOINDEX_NOFOLLOW',
    sitemap_enabled BOOLEAN NOT NULL DEFAULT FALSE,
    provider_generation BIGINT NOT NULL DEFAULT 1,
    navigation_generation BIGINT NOT NULL DEFAULT 1,
    search_generation BIGINT NOT NULL DEFAULT 1,
    last_projected_drive_checkpoint BIGINT NOT NULL DEFAULT 0,
    activated_at TIMESTAMPTZ,
    paused_at TIMESTAMPTZ,
    last_error_code VARCHAR(128),
    created_by BIGINT NOT NULL,
    updated_by BIGINT NOT NULL,
    status SMALLINT NOT NULL DEFAULT 1,
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL,
    version BIGINT NOT NULL DEFAULT 0,
    CONSTRAINT ck_kb_site_publication_tenant CHECK (tenant_id > 0),
    CONSTRAINT ck_kb_site_publication_organization CHECK (organization_id >= 0),
    CONSTRAINT ck_kb_site_publication_type CHECK (publication_type = 'wiki'),
    CONSTRAINT ck_kb_site_publication_state CHECK (wiki_status IN (
        'DRAFT', 'VALIDATING', 'READY', 'ACTIVE', 'DEGRADED', 'PAUSED', 'ARCHIVED', 'FAILED'
    )),
    CONSTRAINT ck_kb_site_publication_mode CHECK (publication_mode IN (
        'REVIEW_REQUIRED', 'AUTO_PUBLIC_AFTER_CHECKS'
    )),
    CONSTRAINT ck_kb_site_publication_visibility CHECK (default_visibility IN (
        'PRIVATE', 'UNLISTED', 'PUBLIC'
    )),
    CONSTRAINT ck_kb_site_publication_update_policy CHECK (update_policy IN (
        'KEEP_LAST_PUBLIC_UNTIL_READY', 'UNPUBLISH_DURING_PROCESSING'
    )),
    CONSTRAINT ck_kb_site_publication_navigation CHECK (navigation_mode IN (
        'DIRECTORY', 'FRONT_MATTER', 'CURATED'
    )),
    CONSTRAINT ck_kb_site_publication_robots CHECK (robots_policy IN (
        'INDEX_FOLLOW', 'NOINDEX_NOFOLLOW'
    )),
    CONSTRAINT ck_kb_site_publication_generation CHECK (
        provider_generation >= 1 AND navigation_generation >= 1
        AND search_generation >= 1 AND last_projected_drive_checkpoint >= 0
    ),
    CONSTRAINT ck_kb_site_publication_ready_root CHECK (
        wiki_status IN ('DRAFT', 'VALIDATING', 'ARCHIVED', 'FAILED')
        OR (source_root_node_uuid IS NOT NULL AND source_scope_uuid IS NOT NULL)
    ),
    CONSTRAINT ck_kb_site_publication_json_bounds CHECK (
        octet_length(supported_locales_json::text) <= 8192
        AND octet_length(navigation_config_json::text) <= 32768
        AND octet_length(theme_config_json::text) <= 32768
    ),
    CONSTRAINT fk_kb_site_publication_space
        FOREIGN KEY (tenant_id, organization_id, space_id)
        REFERENCES kb_space(tenant_id, organization_id, id)
);

CREATE UNIQUE INDEX IF NOT EXISTS uk_kb_site_publication_uuid
    ON kb_site_publication (tenant_id, uuid);
CREATE UNIQUE INDEX IF NOT EXISTS uk_kb_site_publication_scope_id
    ON kb_site_publication (tenant_id, organization_id, id);
CREATE UNIQUE INDEX IF NOT EXISTS uk_kb_site_publication_space
    ON kb_site_publication (tenant_id, space_id);
CREATE UNIQUE INDEX IF NOT EXISTS uk_kb_site_publication_drive_space
    ON kb_site_publication (tenant_id, drive_space_uuid)
    WHERE status = 1;
CREATE INDEX IF NOT EXISTS idx_kb_site_publication_state
    ON kb_site_publication (
        tenant_id, organization_id, wiki_status, updated_at DESC, id DESC
    );

CREATE TABLE IF NOT EXISTS kb_source_file_projection (
    id BIGINT NOT NULL PRIMARY KEY,
    uuid VARCHAR(64) NOT NULL,
    tenant_id BIGINT NOT NULL,
    organization_id BIGINT NOT NULL,
    site_publication_id BIGINT NOT NULL,
    space_id BIGINT NOT NULL,
    drive_space_uuid VARCHAR(64) NOT NULL,
    drive_node_uuid VARCHAR(64) NOT NULL,
    drive_version_uuid VARCHAR(64) NOT NULL,
    source_path VARCHAR(4096) NOT NULL,
    canonical_route VARCHAR(2048),
    file_kind VARCHAR(32) NOT NULL,
    media_type VARCHAR(255) NOT NULL,
    size_bytes BIGINT NOT NULL,
    content_sha256 VARCHAR(71) NOT NULL,
    source_state VARCHAR(32) NOT NULL DEFAULT 'DISCOVERED',
    publication_state VARCHAR(32) NOT NULL DEFAULT 'DRAFT',
    visibility VARCHAR(16) NOT NULL DEFAULT 'PRIVATE',
    index_state VARCHAR(32) NOT NULL DEFAULT 'NOT_REQUIRED',
    title VARCHAR(512),
    description VARCHAR(2048),
    locale VARCHAR(35),
    nav_order INTEGER,
    nav_hidden BOOLEAN NOT NULL DEFAULT FALSE,
    scheduled_publish_at TIMESTAMPTZ,
    published_at TIMESTAMPTZ,
    unpublished_at TIMESTAMPTZ,
    public_drive_version_uuid VARCHAR(64),
    page_public_version BIGINT NOT NULL DEFAULT 0,
    parser_version VARCHAR(64),
    renderer_policy_version VARCHAR(64),
    index_version VARCHAR(64),
    previous_canonical_route VARCHAR(2048),
    redirect_status SMALLINT,
    redirect_expires_at TIMESTAMPTZ,
    source_sequence_no BIGINT NOT NULL DEFAULT 0,
    last_source_event_id VARCHAR(128),
    processing_attempt_count INTEGER NOT NULL DEFAULT 0,
    next_processing_at TIMESTAMPTZ,
    processing_lease_owner VARCHAR(128),
    processing_lease_token VARCHAR(128),
    processing_lease_expires_at TIMESTAMPTZ,
    processing_fence BIGINT NOT NULL DEFAULT 0,
    last_error_code VARCHAR(128),
    last_error_summary VARCHAR(1024),
    last_processed_at TIMESTAMPTZ,
    last_indexed_at TIMESTAMPTZ,
    created_by BIGINT NOT NULL,
    updated_by BIGINT NOT NULL,
    status SMALLINT NOT NULL DEFAULT 1,
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL,
    version BIGINT NOT NULL DEFAULT 0,
    CONSTRAINT ck_kb_source_projection_tenant CHECK (tenant_id > 0),
    CONSTRAINT ck_kb_source_projection_organization CHECK (organization_id >= 0),
    CONSTRAINT ck_kb_source_projection_file_kind CHECK (file_kind IN (
        'PAGE', 'DOCUMENT', 'PRESENTATION', 'SPREADSHEET', 'CODE', 'MEDIA', 'ASSET', 'ARCHIVE'
    )),
    CONSTRAINT ck_kb_source_projection_source_state CHECK (source_state IN (
        'DISCOVERED', 'QUEUED', 'PROCESSING', 'READY', 'ERROR', 'QUARANTINED', 'DELETED'
    )),
    CONSTRAINT ck_kb_source_projection_publication_state CHECK (publication_state IN (
        'DRAFT', 'IN_REVIEW', 'SCHEDULED', 'PUBLISHED', 'UNPUBLISHED', 'ARCHIVED'
    )),
    CONSTRAINT ck_kb_source_projection_visibility CHECK (visibility IN (
        'PRIVATE', 'UNLISTED', 'PUBLIC'
    )),
    CONSTRAINT ck_kb_source_projection_index_state CHECK (index_state IN (
        'NOT_REQUIRED', 'PENDING', 'INDEXING', 'READY', 'ERROR'
    )),
    CONSTRAINT ck_kb_source_projection_hash CHECK (
        content_sha256 ~ '^sha256:[0-9a-f]{64}$'
    ),
    CONSTRAINT ck_kb_source_projection_bounds CHECK (
        size_bytes >= 0 AND page_public_version >= 0 AND source_sequence_no >= 0
        AND processing_attempt_count >= 0 AND processing_fence >= 0
    ),
    CONSTRAINT ck_kb_source_projection_redirect CHECK (
        redirect_status IS NULL OR redirect_status IN (301, 302, 307, 308)
    ),
    CONSTRAINT ck_kb_source_projection_public_version CHECK (
        publication_state <> 'PUBLISHED'
        OR (visibility IN ('UNLISTED', 'PUBLIC')
            AND canonical_route IS NOT NULL AND public_drive_version_uuid IS NOT NULL
            AND page_public_version > 0)
    ),
    CONSTRAINT fk_kb_source_projection_publication
        FOREIGN KEY (tenant_id, organization_id, site_publication_id)
        REFERENCES kb_site_publication(tenant_id, organization_id, id),
    CONSTRAINT fk_kb_source_projection_space
        FOREIGN KEY (tenant_id, organization_id, space_id)
        REFERENCES kb_space(tenant_id, organization_id, id)
);

CREATE UNIQUE INDEX IF NOT EXISTS uk_kb_source_projection_uuid
    ON kb_source_file_projection (tenant_id, uuid);
CREATE UNIQUE INDEX IF NOT EXISTS uk_kb_source_projection_scope_id
    ON kb_source_file_projection (tenant_id, organization_id, id);
CREATE UNIQUE INDEX IF NOT EXISTS uk_kb_source_projection_node
    ON kb_source_file_projection (
        tenant_id, organization_id, site_publication_id, drive_node_uuid
    ) WHERE status = 1;
CREATE UNIQUE INDEX IF NOT EXISTS uk_kb_source_projection_path
    ON kb_source_file_projection (
        tenant_id, organization_id, site_publication_id, source_path
    ) WHERE status = 1 AND source_state <> 'DELETED';
CREATE UNIQUE INDEX IF NOT EXISTS uk_kb_source_projection_public_route
    ON kb_source_file_projection (
        tenant_id, organization_id, site_publication_id, canonical_route
    ) WHERE status = 1 AND publication_state = 'PUBLISHED'
      AND visibility IN ('UNLISTED', 'PUBLIC');
CREATE INDEX IF NOT EXISTS idx_kb_source_projection_state
    ON kb_source_file_projection (
        tenant_id, organization_id, site_publication_id,
        source_state, publication_state, updated_at DESC, id DESC
    );
CREATE INDEX IF NOT EXISTS idx_kb_source_projection_claimable
    ON kb_source_file_projection (
        tenant_id, organization_id, source_state, next_processing_at, updated_at, id
    ) WHERE status = 1 AND source_state IN ('DISCOVERED', 'QUEUED', 'ERROR');
CREATE INDEX IF NOT EXISTS idx_kb_source_projection_scheduled
    ON kb_source_file_projection (
        tenant_id, organization_id, scheduled_publish_at, id
    ) WHERE status = 1 AND publication_state = 'SCHEDULED';
CREATE INDEX IF NOT EXISTS idx_kb_source_projection_public_lookup
    ON kb_source_file_projection (
        tenant_id, organization_id, site_publication_id, canonical_route,
        page_public_version, id
    ) WHERE status = 1 AND publication_state = 'PUBLISHED'
      AND visibility IN ('UNLISTED', 'PUBLIC');

CREATE TABLE IF NOT EXISTS kb_source_file_rendition (
    id BIGINT NOT NULL PRIMARY KEY,
    uuid VARCHAR(64) NOT NULL,
    tenant_id BIGINT NOT NULL,
    organization_id BIGINT NOT NULL,
    site_publication_id BIGINT NOT NULL,
    source_file_projection_id BIGINT NOT NULL,
    drive_version_uuid VARCHAR(64) NOT NULL,
    source_content_sha256 VARCHAR(71) NOT NULL,
    processor_id VARCHAR(128) NOT NULL,
    processor_version VARCHAR(64) NOT NULL,
    policy_version VARCHAR(64) NOT NULL,
    rendition_kind VARCHAR(32) NOT NULL,
    rendition_key_sha256 VARCHAR(71) NOT NULL,
    rendition_state VARCHAR(32) NOT NULL DEFAULT 'PENDING',
    locale VARCHAR(35),
    rendition_drive_space_uuid VARCHAR(64),
    rendition_drive_node_uuid VARCHAR(64),
    rendition_drive_version_uuid VARCHAR(64),
    media_resource_snapshot JSONB,
    content_sha256 VARCHAR(71),
    media_type VARCHAR(255),
    size_bytes BIGINT,
    page_or_slide_count INTEGER,
    width INTEGER,
    height INTEGER,
    duration_millis BIGINT,
    processing_attempt_count INTEGER NOT NULL DEFAULT 0,
    next_processing_at TIMESTAMPTZ,
    processing_lease_owner VARCHAR(128),
    processing_lease_token VARCHAR(128),
    processing_lease_expires_at TIMESTAMPTZ,
    processing_fence BIGINT NOT NULL DEFAULT 0,
    error_code VARCHAR(128),
    error_summary VARCHAR(1024),
    processed_at TIMESTAMPTZ,
    expires_at TIMESTAMPTZ,
    created_by BIGINT NOT NULL,
    updated_by BIGINT NOT NULL,
    status SMALLINT NOT NULL DEFAULT 1,
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL,
    version BIGINT NOT NULL DEFAULT 0,
    CONSTRAINT ck_kb_source_rendition_tenant CHECK (tenant_id > 0),
    CONSTRAINT ck_kb_source_rendition_organization CHECK (organization_id >= 0),
    CONSTRAINT ck_kb_source_rendition_kind CHECK (rendition_kind IN (
        'SANITIZED_HTML', 'PDF', 'PAGE_IMAGE', 'THUMBNAIL', 'POSTER', 'PLAIN_TEXT',
        'SLIDE_TEXT', 'SHEET_PREVIEW', 'ARCHIVE_MANIFEST', 'MEDIA_METADATA'
    )),
    CONSTRAINT ck_kb_source_rendition_state CHECK (rendition_state IN (
        'PENDING', 'PROCESSING', 'READY', 'ERROR', 'QUARANTINED', 'EXPIRED'
    )),
    CONSTRAINT ck_kb_source_rendition_source_hash CHECK (
        source_content_sha256 ~ '^sha256:[0-9a-f]{64}$'
        AND rendition_key_sha256 ~ '^sha256:[0-9a-f]{64}$'
        AND (content_sha256 IS NULL OR content_sha256 ~ '^sha256:[0-9a-f]{64}$')
    ),
    CONSTRAINT ck_kb_source_rendition_bounds CHECK (
        processing_attempt_count >= 0 AND processing_fence >= 0
        AND (size_bytes IS NULL OR size_bytes >= 0)
        AND (page_or_slide_count IS NULL OR page_or_slide_count >= 0)
        AND (width IS NULL OR width >= 0) AND (height IS NULL OR height >= 0)
        AND (duration_millis IS NULL OR duration_millis >= 0)
    ),
    CONSTRAINT ck_kb_source_rendition_ready CHECK (
        rendition_state <> 'READY'
        OR (rendition_drive_space_uuid IS NOT NULL AND rendition_drive_node_uuid IS NOT NULL
            AND rendition_drive_version_uuid IS NOT NULL AND content_sha256 IS NOT NULL
            AND media_type IS NOT NULL AND size_bytes IS NOT NULL AND processed_at IS NOT NULL)
    ),
    CONSTRAINT ck_kb_source_rendition_media_snapshot CHECK (
        media_resource_snapshot IS NULL OR octet_length(media_resource_snapshot::text) <= 32768
    ),
    CONSTRAINT fk_kb_source_rendition_publication
        FOREIGN KEY (tenant_id, organization_id, site_publication_id)
        REFERENCES kb_site_publication(tenant_id, organization_id, id),
    CONSTRAINT fk_kb_source_rendition_projection
        FOREIGN KEY (tenant_id, organization_id, source_file_projection_id)
        REFERENCES kb_source_file_projection(tenant_id, organization_id, id)
);

CREATE UNIQUE INDEX IF NOT EXISTS uk_kb_source_rendition_uuid
    ON kb_source_file_rendition (tenant_id, uuid);
CREATE UNIQUE INDEX IF NOT EXISTS uk_kb_source_rendition_identity
    ON kb_source_file_rendition (
        tenant_id, organization_id, source_file_projection_id, rendition_key_sha256
    ) WHERE status = 1;
CREATE INDEX IF NOT EXISTS idx_kb_source_rendition_claimable
    ON kb_source_file_rendition (
        tenant_id, organization_id, rendition_state, next_processing_at, updated_at, id
    ) WHERE status = 1 AND rendition_state IN ('PENDING', 'ERROR');
CREATE INDEX IF NOT EXISTS idx_kb_source_rendition_source_version
    ON kb_source_file_rendition (
        tenant_id, organization_id, source_file_projection_id,
        drive_version_uuid, rendition_kind, updated_at DESC, id DESC
    );

CREATE TABLE IF NOT EXISTS kb_drive_source_checkpoint (
    id BIGINT NOT NULL PRIMARY KEY,
    uuid VARCHAR(64) NOT NULL,
    tenant_id BIGINT NOT NULL,
    organization_id BIGINT NOT NULL,
    site_publication_id BIGINT NOT NULL,
    drive_space_uuid VARCHAR(64) NOT NULL,
    source_scope_uuid VARCHAR(64) NOT NULL,
    last_sequence_no BIGINT NOT NULL DEFAULT 0,
    last_event_id VARCHAR(128),
    stream_state VARCHAR(32) NOT NULL DEFAULT 'HEALTHY',
    gap_from_sequence_no BIGINT,
    gap_to_sequence_no BIGINT,
    reconciliation_cursor VARCHAR(2048),
    reconciliation_started_at TIMESTAMPTZ,
    reconciliation_completed_at TIMESTAMPTZ,
    lease_owner VARCHAR(128),
    lease_token VARCHAR(128),
    lease_expires_at TIMESTAMPTZ,
    fence_token BIGINT NOT NULL DEFAULT 0,
    last_event_time TIMESTAMPTZ,
    last_observed_at TIMESTAMPTZ,
    last_error_code VARCHAR(128),
    last_error_summary VARCHAR(1024),
    created_by BIGINT NOT NULL,
    updated_by BIGINT NOT NULL,
    status SMALLINT NOT NULL DEFAULT 1,
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL,
    version BIGINT NOT NULL DEFAULT 0,
    CONSTRAINT ck_kb_drive_checkpoint_tenant CHECK (tenant_id > 0),
    CONSTRAINT ck_kb_drive_checkpoint_organization CHECK (organization_id >= 0),
    CONSTRAINT ck_kb_drive_checkpoint_state CHECK (stream_state IN (
        'HEALTHY', 'GAP_DETECTED', 'RECONCILING', 'PAUSED', 'FAILED'
    )),
    CONSTRAINT ck_kb_drive_checkpoint_sequence CHECK (
        last_sequence_no >= 0 AND fence_token >= 0
        AND (gap_from_sequence_no IS NULL OR gap_from_sequence_no > last_sequence_no)
        AND (gap_to_sequence_no IS NULL OR gap_to_sequence_no >= gap_from_sequence_no)
    ),
    CONSTRAINT fk_kb_drive_checkpoint_publication
        FOREIGN KEY (tenant_id, organization_id, site_publication_id)
        REFERENCES kb_site_publication(tenant_id, organization_id, id)
);

CREATE UNIQUE INDEX IF NOT EXISTS uk_kb_drive_checkpoint_uuid
    ON kb_drive_source_checkpoint (tenant_id, uuid);
CREATE UNIQUE INDEX IF NOT EXISTS uk_kb_drive_checkpoint_scope_id
    ON kb_drive_source_checkpoint (tenant_id, organization_id, id);
CREATE UNIQUE INDEX IF NOT EXISTS uk_kb_drive_checkpoint_publication
    ON kb_drive_source_checkpoint (tenant_id, organization_id, site_publication_id);
CREATE UNIQUE INDEX IF NOT EXISTS uk_kb_drive_checkpoint_source_scope
    ON kb_drive_source_checkpoint (
        tenant_id, organization_id, drive_space_uuid, source_scope_uuid
    );
CREATE INDEX IF NOT EXISTS idx_kb_drive_checkpoint_reconcile
    ON kb_drive_source_checkpoint (
        tenant_id, organization_id, stream_state, lease_expires_at, updated_at, id
    ) WHERE status = 1 AND stream_state IN ('GAP_DETECTED', 'RECONCILING', 'FAILED');

CREATE TABLE IF NOT EXISTS kb_drive_event_inbox (
    id BIGINT NOT NULL PRIMARY KEY,
    uuid VARCHAR(64) NOT NULL,
    tenant_id BIGINT NOT NULL,
    organization_id BIGINT NOT NULL,
    site_publication_id BIGINT NOT NULL,
    checkpoint_id BIGINT NOT NULL,
    source_event_id VARCHAR(128) NOT NULL,
    event_type VARCHAR(128) NOT NULL,
    sequence_no BIGINT NOT NULL,
    drive_node_uuid VARCHAR(64) NOT NULL,
    drive_version_uuid VARCHAR(64),
    payload_sha256 VARCHAR(71) NOT NULL,
    payload_json JSONB NOT NULL,
    source_event_time TIMESTAMPTZ NOT NULL,
    processing_state VARCHAR(32) NOT NULL DEFAULT 'RECEIVED',
    attempt_count INTEGER NOT NULL DEFAULT 0,
    next_retry_at TIMESTAMPTZ,
    lease_owner VARCHAR(128),
    lease_token VARCHAR(128),
    lease_expires_at TIMESTAMPTZ,
    last_error_code VARCHAR(128),
    last_error_summary VARCHAR(1024),
    received_at TIMESTAMPTZ NOT NULL,
    applied_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL,
    version BIGINT NOT NULL DEFAULT 0,
    CONSTRAINT ck_kb_drive_inbox_tenant CHECK (tenant_id > 0),
    CONSTRAINT ck_kb_drive_inbox_organization CHECK (organization_id >= 0),
    CONSTRAINT ck_kb_drive_inbox_event_type CHECK (event_type IN (
        'drive.node.version.committed.v1', 'drive.node.path.changed.v1',
        'drive.node.eligibility.changed.v1', 'drive.node.deleted.v1'
    )),
    CONSTRAINT ck_kb_drive_inbox_state CHECK (processing_state IN (
        'RECEIVED', 'DEFERRED', 'APPLIED', 'RETRY', 'DEAD_LETTER', 'IGNORED'
    )),
    CONSTRAINT ck_kb_drive_inbox_bounds CHECK (
        sequence_no >= 1 AND attempt_count >= 0 AND octet_length(payload_json::text) <= 65536
    ),
    CONSTRAINT ck_kb_drive_inbox_hash CHECK (
        payload_sha256 ~ '^sha256:[0-9a-f]{64}$'
    ),
    CONSTRAINT fk_kb_drive_inbox_publication
        FOREIGN KEY (tenant_id, organization_id, site_publication_id)
        REFERENCES kb_site_publication(tenant_id, organization_id, id),
    CONSTRAINT fk_kb_drive_inbox_checkpoint
        FOREIGN KEY (tenant_id, organization_id, checkpoint_id)
        REFERENCES kb_drive_source_checkpoint(tenant_id, organization_id, id)
);

CREATE UNIQUE INDEX IF NOT EXISTS uk_kb_drive_inbox_uuid
    ON kb_drive_event_inbox (tenant_id, uuid);
CREATE UNIQUE INDEX IF NOT EXISTS uk_kb_drive_inbox_event
    ON kb_drive_event_inbox (
        tenant_id, organization_id, site_publication_id, source_event_id
    );
CREATE UNIQUE INDEX IF NOT EXISTS uk_kb_drive_inbox_sequence
    ON kb_drive_event_inbox (
        tenant_id, organization_id, checkpoint_id, sequence_no
    );
CREATE INDEX IF NOT EXISTS idx_kb_drive_inbox_apply
    ON kb_drive_event_inbox (
        tenant_id, organization_id, checkpoint_id, processing_state,
        sequence_no, id
    );
CREATE INDEX IF NOT EXISTS idx_kb_drive_inbox_retry
    ON kb_drive_event_inbox (
        tenant_id, organization_id, processing_state, next_retry_at, sequence_no, id
    ) WHERE processing_state IN ('RECEIVED', 'RETRY', 'DEFERRED');

DO $$
DECLARE
    table_name text;
BEGIN
    FOREACH table_name IN ARRAY ARRAY[
        'kb_site_publication',
        'kb_source_file_projection',
        'kb_source_file_rendition',
        'kb_drive_source_checkpoint',
        'kb_drive_event_inbox'
    ]
    LOOP
        EXECUTE format('ALTER TABLE %I ENABLE ROW LEVEL SECURITY', table_name);
        EXECUTE format('ALTER TABLE %I FORCE ROW LEVEL SECURITY', table_name);
        IF NOT EXISTS (
            SELECT 1 FROM pg_policies
            WHERE schemaname = current_schema()
              AND tablename = table_name
              AND policyname = 'tenant_isolation'
        ) THEN
            EXECUTE format(
                'CREATE POLICY tenant_isolation ON %I AS PERMISSIVE FOR ALL TO PUBLIC USING (tenant_id = current_setting(''app.current_tenant_id'', true)::bigint) WITH CHECK (tenant_id = current_setting(''app.current_tenant_id'', true)::bigint)',
                table_name
            );
        END IF;
    END LOOP;
END $$;
