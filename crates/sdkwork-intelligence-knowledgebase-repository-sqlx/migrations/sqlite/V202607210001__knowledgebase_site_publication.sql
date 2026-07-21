CREATE TABLE IF NOT EXISTS kb_site (
    id BIGINT NOT NULL,
    uuid TEXT NOT NULL,
    tenant_id BIGINT NOT NULL,
    organization_id BIGINT NOT NULL,
    space_id BIGINT NOT NULL,
    title TEXT NOT NULL,
    visibility TEXT NOT NULL,
    homepage_concept_id TEXT,
    theme_id TEXT NOT NULL,
    publish_mode TEXT NOT NULL,
    lifecycle_state TEXT NOT NULL,
    canonical_host_binding_id BIGINT,
    current_release_id BIGINT,
    status INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    version BIGINT NOT NULL DEFAULT 0,
    PRIMARY KEY (id),
    FOREIGN KEY (space_id) REFERENCES kb_space(id),
    CHECK (visibility IN ('private', 'unlisted', 'public')),
    CHECK (publish_mode IN ('manual', 'automatic')),
    CHECK (lifecycle_state IN ('draft', 'active', 'paused')),
    CHECK (status IN (0, 1))
);

CREATE UNIQUE INDEX IF NOT EXISTS uk_kb_site_uuid
    ON kb_site (tenant_id, uuid);
CREATE UNIQUE INDEX IF NOT EXISTS uk_kb_site_space
    ON kb_site (tenant_id, organization_id, space_id)
    WHERE status = 1;
CREATE INDEX IF NOT EXISTS idx_kb_site_public_space
    ON kb_site (tenant_id, space_id, lifecycle_state, visibility, current_release_id)
    WHERE status = 1;

CREATE TABLE IF NOT EXISTS kb_site_release (
    id BIGINT NOT NULL,
    uuid TEXT NOT NULL,
    tenant_id BIGINT NOT NULL,
    organization_id BIGINT NOT NULL,
    site_id BIGINT NOT NULL,
    lifecycle_state TEXT NOT NULL,
    source_content_hash TEXT NOT NULL,
    manifest_drive_uri TEXT,
    manifest_drive_space_id TEXT,
    manifest_drive_node_id TEXT,
    manifest_checksum_sha256_hex TEXT,
    page_count INTEGER NOT NULL DEFAULT 0,
    asset_count INTEGER NOT NULL DEFAULT 0,
    previous_release_id BIGINT,
    error_code TEXT,
    status INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL,
    completed_at TEXT,
    version BIGINT NOT NULL DEFAULT 0,
    PRIMARY KEY (id),
    FOREIGN KEY (site_id) REFERENCES kb_site(id),
    CHECK (lifecycle_state IN ('building', 'ready', 'failed')),
    CHECK (page_count >= 0),
    CHECK (asset_count >= 0),
    CHECK (status IN (0, 1)),
    CHECK (
        lifecycle_state <> 'ready'
        OR (
            manifest_drive_uri IS NOT NULL
            AND manifest_drive_space_id IS NOT NULL
            AND manifest_drive_node_id IS NOT NULL
            AND manifest_checksum_sha256_hex IS NOT NULL
            AND completed_at IS NOT NULL
        )
    )
);

CREATE UNIQUE INDEX IF NOT EXISTS uk_kb_site_release_uuid
    ON kb_site_release (tenant_id, uuid);
CREATE UNIQUE INDEX IF NOT EXISTS uk_kb_site_release_content
    ON kb_site_release (tenant_id, site_id, source_content_hash)
    WHERE status = 1;
CREATE INDEX IF NOT EXISTS idx_kb_site_release_history
    ON kb_site_release (tenant_id, site_id, id DESC)
    WHERE status = 1;

CREATE TABLE IF NOT EXISTS kb_site_host_binding (
    id BIGINT NOT NULL,
    uuid TEXT NOT NULL,
    tenant_id BIGINT NOT NULL,
    organization_id BIGINT NOT NULL,
    site_id BIGINT NOT NULL,
    binding_type TEXT NOT NULL,
    normalized_host TEXT NOT NULL,
    canonical INTEGER NOT NULL DEFAULT 0,
    lifecycle_state TEXT NOT NULL,
    web_server_site_id TEXT,
    web_server_domain_id TEXT,
    web_server_deployment_id TEXT,
    status INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    version BIGINT NOT NULL DEFAULT 0,
    PRIMARY KEY (id),
    FOREIGN KEY (site_id) REFERENCES kb_site(id),
    CHECK (binding_type IN ('system_id', 'custom_prefix', 'external_domain')),
    CHECK (canonical IN (0, 1)),
    CHECK (lifecycle_state IN ('pending', 'verified', 'active', 'failed')),
    CHECK (normalized_host = lower(normalized_host)),
    CHECK (status IN (0, 1))
);

CREATE UNIQUE INDEX IF NOT EXISTS uk_kb_site_host_binding_uuid
    ON kb_site_host_binding (tenant_id, uuid);
CREATE UNIQUE INDEX IF NOT EXISTS uk_kb_site_host_binding_host
    ON kb_site_host_binding (normalized_host)
    WHERE status = 1;
CREATE UNIQUE INDEX IF NOT EXISTS uk_kb_site_host_binding_type
    ON kb_site_host_binding (tenant_id, site_id, binding_type)
    WHERE status = 1 AND binding_type IN ('system_id', 'custom_prefix');
CREATE UNIQUE INDEX IF NOT EXISTS uk_kb_site_host_binding_canonical
    ON kb_site_host_binding (tenant_id, site_id)
    WHERE status = 1 AND canonical = 1;
CREATE INDEX IF NOT EXISTS idx_kb_site_host_binding_site
    ON kb_site_host_binding (tenant_id, site_id, id)
    WHERE status = 1;
