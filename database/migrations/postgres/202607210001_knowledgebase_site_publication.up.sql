CREATE TABLE IF NOT EXISTS kb_site (
    id BIGINT NOT NULL,
    uuid VARCHAR(64) NOT NULL,
    tenant_id BIGINT NOT NULL,
    organization_id BIGINT NOT NULL,
    space_id BIGINT NOT NULL,
    title VARCHAR(256) NOT NULL,
    visibility VARCHAR(32) NOT NULL,
    homepage_concept_id VARCHAR(512),
    theme_id VARCHAR(64) NOT NULL,
    publish_mode VARCHAR(32) NOT NULL,
    lifecycle_state VARCHAR(32) NOT NULL,
    canonical_host_binding_id BIGINT,
    current_release_id BIGINT,
    status SMALLINT NOT NULL DEFAULT 1,
    created_at TIMESTAMP NOT NULL,
    updated_at TIMESTAMP NOT NULL,
    version BIGINT NOT NULL DEFAULT 0,
    PRIMARY KEY (id),
    FOREIGN KEY (space_id) REFERENCES kb_space(id),
    CONSTRAINT ck_kb_site_visibility CHECK (visibility IN ('private', 'unlisted', 'public')),
    CONSTRAINT ck_kb_site_publish_mode CHECK (publish_mode IN ('manual', 'automatic')),
    CONSTRAINT ck_kb_site_lifecycle CHECK (lifecycle_state IN ('draft', 'active', 'paused')),
    CONSTRAINT ck_kb_site_status CHECK (status IN (0, 1))
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
    uuid VARCHAR(64) NOT NULL,
    tenant_id BIGINT NOT NULL,
    organization_id BIGINT NOT NULL,
    site_id BIGINT NOT NULL,
    lifecycle_state VARCHAR(32) NOT NULL,
    source_content_hash VARCHAR(64) NOT NULL,
    manifest_drive_uri VARCHAR(1024),
    manifest_drive_space_id VARCHAR(128),
    manifest_drive_node_id VARCHAR(128),
    manifest_checksum_sha256_hex VARCHAR(64),
    page_count INTEGER NOT NULL DEFAULT 0,
    asset_count INTEGER NOT NULL DEFAULT 0,
    previous_release_id BIGINT,
    error_code VARCHAR(128),
    status SMALLINT NOT NULL DEFAULT 1,
    created_at TIMESTAMP NOT NULL,
    completed_at TIMESTAMP,
    version BIGINT NOT NULL DEFAULT 0,
    PRIMARY KEY (id),
    FOREIGN KEY (site_id) REFERENCES kb_site(id),
    CONSTRAINT ck_kb_site_release_lifecycle
        CHECK (lifecycle_state IN ('building', 'ready', 'failed')),
    CONSTRAINT ck_kb_site_release_counts CHECK (page_count >= 0 AND asset_count >= 0),
    CONSTRAINT ck_kb_site_release_status CHECK (status IN (0, 1)),
    CONSTRAINT ck_kb_site_release_ready_manifest CHECK (
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
    uuid VARCHAR(64) NOT NULL,
    tenant_id BIGINT NOT NULL,
    organization_id BIGINT NOT NULL,
    site_id BIGINT NOT NULL,
    binding_type VARCHAR(32) NOT NULL,
    normalized_host VARCHAR(253) NOT NULL,
    canonical SMALLINT NOT NULL DEFAULT 0,
    lifecycle_state VARCHAR(32) NOT NULL,
    web_server_site_id VARCHAR(128),
    web_server_domain_id VARCHAR(128),
    web_server_deployment_id VARCHAR(128),
    status SMALLINT NOT NULL DEFAULT 1,
    created_at TIMESTAMP NOT NULL,
    updated_at TIMESTAMP NOT NULL,
    version BIGINT NOT NULL DEFAULT 0,
    PRIMARY KEY (id),
    FOREIGN KEY (site_id) REFERENCES kb_site(id),
    CONSTRAINT ck_kb_site_host_binding_type
        CHECK (binding_type IN ('system_id', 'custom_prefix', 'external_domain')),
    CONSTRAINT ck_kb_site_host_binding_lifecycle
        CHECK (lifecycle_state IN ('pending', 'verified', 'active', 'failed')),
    CONSTRAINT ck_kb_site_host_binding_canonical CHECK (canonical IN (0, 1)),
    CONSTRAINT ck_kb_site_host_binding_normalized
        CHECK (normalized_host = lower(normalized_host)),
    CONSTRAINT ck_kb_site_host_binding_status CHECK (status IN (0, 1))
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

DO $$
DECLARE
    table_name text;
BEGIN
    FOR table_name IN
        SELECT unnest(ARRAY['kb_site', 'kb_site_release', 'kb_site_host_binding'])
    LOOP
        EXECUTE format('ALTER TABLE %I ENABLE ROW LEVEL SECURITY', table_name);
        EXECUTE format('ALTER TABLE %I FORCE ROW LEVEL SECURITY', table_name);
        IF NOT EXISTS (
            SELECT 1
            FROM pg_policies
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
