-- source: sdkwork-web-framework/crates/sdkwork-web-store-sqlx/migrations/003_web_audit_event.sql
-- source: sdkwork-web-framework/crates/sdkwork-web-store-sqlx/migrations/009_web_audit_outcome.sql

CREATE TABLE IF NOT EXISTS web_audit_event (
    id BIGSERIAL PRIMARY KEY,
    request_id VARCHAR(128) NOT NULL,
    tenant_id VARCHAR(128),
    user_id VARCHAR(128),
    api_surface VARCHAR(64) NOT NULL,
    path TEXT NOT NULL,
    method VARCHAR(16) NOT NULL,
    operation_id VARCHAR(128),
    status_code INTEGER,
    duration_ms INTEGER,
    created_at BIGINT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_web_audit_event_created
    ON web_audit_event (created_at);

CREATE INDEX IF NOT EXISTS idx_web_audit_event_request
    ON web_audit_event (request_id);

CREATE INDEX IF NOT EXISTS idx_web_audit_event_tenant
    ON web_audit_event (tenant_id);
