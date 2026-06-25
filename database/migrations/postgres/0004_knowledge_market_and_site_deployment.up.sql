-- source: crates/sdkwork-intelligence-knowledgebase-repository-sqlx/migrations/postgres/V202606240001__knowledge_market_and_site_deployment.sql

CREATE TABLE IF NOT EXISTS kb_market_listing (
    id BIGINT NOT NULL,
    tenant_id BIGINT NOT NULL,
    space_id BIGINT NOT NULL,
    title VARCHAR(256) NOT NULL,
    icon VARCHAR(64),
    description TEXT,
    author VARCHAR(128),
    tags_json TEXT NOT NULL DEFAULT '[]',
    provider VARCHAR(128),
    model_name VARCHAR(128),
    subscribers_count INTEGER NOT NULL DEFAULT 0,
    documents_count INTEGER NOT NULL DEFAULT 0,
    status INTEGER NOT NULL DEFAULT 1,
    created_at TIMESTAMP NOT NULL,
    updated_at TIMESTAMP NOT NULL,
    version INTEGER NOT NULL DEFAULT 0,
    PRIMARY KEY (id),
    FOREIGN KEY (space_id) REFERENCES kb_space(id)
);

CREATE UNIQUE INDEX IF NOT EXISTS uk_kb_market_listing_space
    ON kb_market_listing (tenant_id, space_id)
    WHERE status = 1;

CREATE INDEX IF NOT EXISTS idx_kb_market_listing_status
    ON kb_market_listing (tenant_id, status, updated_at);

CREATE TABLE IF NOT EXISTS kb_market_subscription (
    id BIGINT NOT NULL,
    tenant_id BIGINT NOT NULL,
    subscriber_actor_id BIGINT NOT NULL,
    listing_id BIGINT NOT NULL,
    created_at TIMESTAMP NOT NULL,
    status INTEGER NOT NULL DEFAULT 1,
    PRIMARY KEY (id),
    FOREIGN KEY (listing_id) REFERENCES kb_market_listing(id)
);

CREATE UNIQUE INDEX IF NOT EXISTS uk_kb_market_subscription_actor_listing
    ON kb_market_subscription (tenant_id, subscriber_actor_id, listing_id)
    WHERE status = 1;

CREATE TABLE IF NOT EXISTS kb_site_deployment (
    id BIGINT NOT NULL,
    tenant_id BIGINT NOT NULL,
    space_id BIGINT NOT NULL,
    platform VARCHAR(64) NOT NULL,
    site_name VARCHAR(256),
    custom_domain VARCHAR(256),
    site_logo_data_url TEXT,
    deployed_url TEXT NOT NULL,
    preview_object_key TEXT NOT NULL,
    status INTEGER NOT NULL DEFAULT 1,
    created_at TIMESTAMP NOT NULL,
    updated_at TIMESTAMP NOT NULL,
    version INTEGER NOT NULL DEFAULT 0,
    PRIMARY KEY (id),
    FOREIGN KEY (space_id) REFERENCES kb_space(id)
);

CREATE INDEX IF NOT EXISTS idx_kb_site_deployment_space
    ON kb_site_deployment (tenant_id, space_id, status, updated_at);
