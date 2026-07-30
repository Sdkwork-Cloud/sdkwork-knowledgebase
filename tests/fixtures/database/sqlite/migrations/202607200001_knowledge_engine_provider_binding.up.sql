CREATE UNIQUE INDEX IF NOT EXISTS uk_kb_space_provider_scope
    ON kb_space (tenant_id, organization_id, id);

CREATE TABLE IF NOT EXISTS kb_provider_credential_reference (
    id BIGINT NOT NULL,
    uuid TEXT NOT NULL,
    tenant_id BIGINT NOT NULL,
    organization_id BIGINT NOT NULL,
    implementation_id TEXT NOT NULL,
    display_name TEXT NOT NULL,
    reference_locator TEXT NOT NULL,
    reference_fingerprint TEXT NOT NULL,
    rotation_state TEXT NOT NULL DEFAULT 'current',
    last_rotated_at TEXT,
    created_by TEXT NOT NULL,
    updated_by TEXT NOT NULL,
    status INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    version BIGINT NOT NULL DEFAULT 0,
    PRIMARY KEY (id),
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
    id BIGINT NOT NULL,
    uuid TEXT NOT NULL,
    tenant_id BIGINT NOT NULL,
    organization_id BIGINT NOT NULL,
    space_id BIGINT NOT NULL,
    implementation_id TEXT NOT NULL,
    remote_resource_type TEXT NOT NULL,
    remote_resource_id TEXT NOT NULL,
    credential_reference_id BIGINT,
    lifecycle_state TEXT NOT NULL DEFAULT 'draft',
    capability_snapshot TEXT NOT NULL DEFAULT '[]',
    capability_snapshot_version BIGINT NOT NULL DEFAULT 0,
    last_tested_at TEXT,
    activated_at TEXT,
    disabled_at TEXT,
    last_error_category TEXT,
    created_by TEXT NOT NULL,
    updated_by TEXT NOT NULL,
    status INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    version BIGINT NOT NULL DEFAULT 0,
    PRIMARY KEY (id),
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
    id BIGINT NOT NULL,
    uuid TEXT NOT NULL,
    tenant_id BIGINT NOT NULL,
    organization_id BIGINT NOT NULL,
    space_id BIGINT NOT NULL,
    source_binding_id BIGINT NOT NULL,
    target_binding_id BIGINT NOT NULL,
    operation_state TEXT NOT NULL DEFAULT 'dry_run',
    idempotency_key TEXT NOT NULL,
    requested_by TEXT NOT NULL,
    checkpoint TEXT NOT NULL DEFAULT '{}',
    claim_owner TEXT,
    claim_token TEXT,
    lease_expires_at TEXT,
    attempt_count INTEGER NOT NULL DEFAULT 0,
    cutover_at TEXT,
    observation_until TEXT,
    completed_at TEXT,
    last_error_category TEXT,
    status INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    version BIGINT NOT NULL DEFAULT 0,
    PRIMARY KEY (id),
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
