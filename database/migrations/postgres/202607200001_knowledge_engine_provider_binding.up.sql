CREATE UNIQUE INDEX IF NOT EXISTS uk_kb_space_provider_scope
    ON kb_space (tenant_id, organization_id, id);

CREATE TABLE IF NOT EXISTS kb_provider_credential_reference (
    id BIGINT PRIMARY KEY,
    uuid VARCHAR(64) NOT NULL,
    tenant_id BIGINT NOT NULL,
    organization_id BIGINT NOT NULL,
    implementation_id VARCHAR(128) NOT NULL,
    display_name VARCHAR(256) NOT NULL,
    reference_locator TEXT NOT NULL,
    reference_fingerprint VARCHAR(64) NOT NULL,
    rotation_state VARCHAR(32) NOT NULL DEFAULT 'current',
    last_rotated_at TIMESTAMP,
    created_by VARCHAR(128) NOT NULL,
    updated_by VARCHAR(128) NOT NULL,
    status INTEGER NOT NULL DEFAULT 1,
    created_at TIMESTAMP NOT NULL,
    updated_at TIMESTAMP NOT NULL,
    version BIGINT NOT NULL DEFAULT 0,
    CHECK (tenant_id > 0),
    CHECK (organization_id >= 0),
    CHECK (length(trim(implementation_id)) > 0),
    CHECK (length(trim(reference_locator)) > 0),
    CHECK (rotation_state IN ('current', 'rotation_due', 'revoked'))
);

CREATE UNIQUE INDEX IF NOT EXISTS uk_kb_provider_credential_reference_uuid
    ON kb_provider_credential_reference (tenant_id, organization_id, uuid);

CREATE UNIQUE INDEX IF NOT EXISTS uk_kb_provider_credential_reference_scope_id
    ON kb_provider_credential_reference (tenant_id, organization_id, id);

CREATE UNIQUE INDEX IF NOT EXISTS uk_kb_provider_credential_reference_active
    ON kb_provider_credential_reference (
        tenant_id,
        organization_id,
        implementation_id,
        reference_fingerprint
    )
    WHERE status = 1;

CREATE TABLE IF NOT EXISTS kb_provider_binding (
    id BIGINT PRIMARY KEY,
    uuid VARCHAR(64) NOT NULL,
    tenant_id BIGINT NOT NULL,
    organization_id BIGINT NOT NULL,
    space_id BIGINT NOT NULL,
    implementation_id VARCHAR(128) NOT NULL,
    remote_resource_type VARCHAR(64) NOT NULL,
    remote_resource_id VARCHAR(512) NOT NULL,
    credential_reference_id BIGINT,
    lifecycle_state VARCHAR(32) NOT NULL DEFAULT 'draft',
    capability_snapshot JSONB NOT NULL DEFAULT '[]'::jsonb,
    capability_snapshot_version BIGINT NOT NULL DEFAULT 0,
    last_tested_at TIMESTAMP,
    activated_at TIMESTAMP,
    disabled_at TIMESTAMP,
    last_error_category VARCHAR(64),
    created_by VARCHAR(128) NOT NULL,
    updated_by VARCHAR(128) NOT NULL,
    status INTEGER NOT NULL DEFAULT 1,
    created_at TIMESTAMP NOT NULL,
    updated_at TIMESTAMP NOT NULL,
    version BIGINT NOT NULL DEFAULT 0,
    CHECK (tenant_id > 0),
    CHECK (organization_id >= 0),
    CHECK (length(trim(implementation_id)) > 0),
    CHECK (length(trim(remote_resource_type)) > 0),
    CHECK (length(trim(remote_resource_id)) > 0),
    CHECK (lifecycle_state IN ('draft', 'testing', 'active', 'degraded', 'disabled', 'failed')),
    FOREIGN KEY (tenant_id, organization_id, space_id)
        REFERENCES kb_space(tenant_id, organization_id, id),
    FOREIGN KEY (tenant_id, organization_id, credential_reference_id)
        REFERENCES kb_provider_credential_reference(tenant_id, organization_id, id)
);

CREATE UNIQUE INDEX IF NOT EXISTS uk_kb_provider_binding_uuid
    ON kb_provider_binding (tenant_id, organization_id, uuid);

CREATE UNIQUE INDEX IF NOT EXISTS uk_kb_provider_binding_scope_id
    ON kb_provider_binding (tenant_id, organization_id, id);

CREATE UNIQUE INDEX IF NOT EXISTS uk_kb_provider_binding_remote_resource
    ON kb_provider_binding (
        tenant_id,
        organization_id,
        space_id,
        implementation_id,
        remote_resource_type,
        remote_resource_id
    )
    WHERE status = 1;

CREATE UNIQUE INDEX IF NOT EXISTS uk_kb_provider_binding_active_space
    ON kb_provider_binding (tenant_id, organization_id, space_id)
    WHERE lifecycle_state = 'active' AND status = 1;

CREATE INDEX IF NOT EXISTS idx_kb_provider_binding_space_lifecycle
    ON kb_provider_binding (
        tenant_id,
        organization_id,
        space_id,
        lifecycle_state,
        updated_at DESC,
        id DESC
    );

CREATE TABLE IF NOT EXISTS kb_provider_migration_operation (
    id BIGINT PRIMARY KEY,
    uuid VARCHAR(64) NOT NULL,
    tenant_id BIGINT NOT NULL,
    organization_id BIGINT NOT NULL,
    space_id BIGINT NOT NULL,
    source_binding_id BIGINT NOT NULL,
    target_binding_id BIGINT NOT NULL,
    operation_state VARCHAR(32) NOT NULL DEFAULT 'dry_run',
    idempotency_key VARCHAR(128) NOT NULL,
    requested_by VARCHAR(128) NOT NULL,
    checkpoint JSONB NOT NULL DEFAULT '{}'::jsonb,
    claim_owner VARCHAR(128),
    claim_token VARCHAR(64),
    lease_expires_at TIMESTAMP,
    attempt_count INTEGER NOT NULL DEFAULT 0,
    cutover_at TIMESTAMP,
    observation_until TIMESTAMP,
    completed_at TIMESTAMP,
    last_error_category VARCHAR(64),
    status INTEGER NOT NULL DEFAULT 1,
    created_at TIMESTAMP NOT NULL,
    updated_at TIMESTAMP NOT NULL,
    version BIGINT NOT NULL DEFAULT 0,
    CHECK (tenant_id > 0),
    CHECK (organization_id >= 0),
    CHECK (source_binding_id <> target_binding_id),
    CHECK (operation_state IN (
        'dry_run', 'preparing', 'validating', 'cutover', 'observing',
        'completed', 'rolling_back', 'rolled_back', 'failed'
    )),
    FOREIGN KEY (tenant_id, organization_id, space_id)
        REFERENCES kb_space(tenant_id, organization_id, id),
    FOREIGN KEY (tenant_id, organization_id, source_binding_id)
        REFERENCES kb_provider_binding(tenant_id, organization_id, id),
    FOREIGN KEY (tenant_id, organization_id, target_binding_id)
        REFERENCES kb_provider_binding(tenant_id, organization_id, id)
);

CREATE UNIQUE INDEX IF NOT EXISTS uk_kb_provider_migration_operation_uuid
    ON kb_provider_migration_operation (tenant_id, organization_id, uuid);

CREATE UNIQUE INDEX IF NOT EXISTS uk_kb_provider_migration_operation_idempotency
    ON kb_provider_migration_operation (tenant_id, organization_id, idempotency_key);

CREATE UNIQUE INDEX IF NOT EXISTS uk_kb_provider_migration_operation_active_space
    ON kb_provider_migration_operation (tenant_id, organization_id, space_id)
    WHERE operation_state NOT IN ('completed', 'rolled_back', 'failed') AND status = 1;

CREATE INDEX IF NOT EXISTS idx_kb_provider_migration_operation_claimable
    ON kb_provider_migration_operation (
        tenant_id,
        organization_id,
        operation_state,
        lease_expires_at,
        updated_at,
        id
    );

DO $$
DECLARE
    table_name text;
BEGIN
    FOR table_name IN
        SELECT unnest(ARRAY[
            'kb_provider_credential_reference',
            'kb_provider_binding',
            'kb_provider_migration_operation'
        ])
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
