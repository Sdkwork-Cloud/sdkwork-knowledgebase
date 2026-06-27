-- source: crates/sdkwork-intelligence-knowledgebase-repository-sqlx/migrations/sqlite/V202606240001__knowledge_market_and_site_deployment.sql

CREATE TABLE IF NOT EXISTS kb_market_listing (
    id BIGINT NOT NULL,
    tenant_id INTEGER NOT NULL,
    space_id BIGINT NOT NULL,
    title TEXT NOT NULL,
    icon TEXT,
    description TEXT,
    author TEXT,
    tags_json TEXT NOT NULL DEFAULT '[]',
    provider TEXT,
    model_name TEXT,
    subscribers_count INTEGER NOT NULL DEFAULT 0,
    documents_count INTEGER NOT NULL DEFAULT 0,
    status INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
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
    tenant_id INTEGER NOT NULL,
    subscriber_actor_id BIGINT NOT NULL,
    listing_id BIGINT NOT NULL,
    created_at TEXT NOT NULL,
    status INTEGER NOT NULL DEFAULT 1,
    PRIMARY KEY (id),
    FOREIGN KEY (listing_id) REFERENCES kb_market_listing(id)
);

CREATE UNIQUE INDEX IF NOT EXISTS uk_kb_market_subscription_actor_listing
    ON kb_market_subscription (tenant_id, subscriber_actor_id, listing_id)
    WHERE status = 1;

CREATE TABLE IF NOT EXISTS kb_site_deployment (
    id BIGINT NOT NULL,
    tenant_id INTEGER NOT NULL,
    space_id BIGINT NOT NULL,
    platform TEXT NOT NULL,
    site_name TEXT,
    custom_domain TEXT,
    site_logo_data_url TEXT,
    deployed_url TEXT NOT NULL,
    preview_object_key TEXT NOT NULL,
    status INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    version INTEGER NOT NULL DEFAULT 0,
    PRIMARY KEY (id),
    FOREIGN KEY (space_id) REFERENCES kb_space(id)
);

CREATE INDEX IF NOT EXISTS idx_kb_site_deployment_space
    ON kb_site_deployment (tenant_id, space_id, status, updated_at);
