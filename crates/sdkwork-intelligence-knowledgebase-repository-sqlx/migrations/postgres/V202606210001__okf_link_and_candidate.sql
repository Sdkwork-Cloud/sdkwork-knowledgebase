CREATE TABLE IF NOT EXISTS kb_okf_concept_link (
    id BIGINT NOT NULL,
    uuid VARCHAR(36) NOT NULL,
    tenant_id BIGINT NOT NULL,
    space_id BIGINT NOT NULL,
    from_concept_id VARCHAR(256) NOT NULL,
    to_concept_id VARCHAR(256) NOT NULL,
    anchor_text VARCHAR(512) NOT NULL DEFAULT '',
    status INTEGER NOT NULL,
    created_at TIMESTAMP NOT NULL,
    updated_at TIMESTAMP NOT NULL,
    version BIGINT NOT NULL DEFAULT 0,
    PRIMARY KEY (id)
);

CREATE UNIQUE INDEX IF NOT EXISTS uk_kb_okf_concept_link_uuid
    ON kb_okf_concept_link (tenant_id, uuid);

CREATE UNIQUE INDEX IF NOT EXISTS uk_kb_okf_concept_link_edge
    ON kb_okf_concept_link (tenant_id, space_id, from_concept_id, to_concept_id, anchor_text);

CREATE INDEX IF NOT EXISTS idx_kb_okf_concept_link_space_from
    ON kb_okf_concept_link (tenant_id, space_id, from_concept_id);

CREATE INDEX IF NOT EXISTS idx_kb_okf_concept_link_space_to
    ON kb_okf_concept_link (tenant_id, space_id, to_concept_id);

CREATE TABLE IF NOT EXISTS kb_okf_candidate (
    id BIGINT NOT NULL,
    uuid VARCHAR(36) NOT NULL,
    tenant_id BIGINT NOT NULL,
    space_id BIGINT NOT NULL,
    concept_id VARCHAR(256) NOT NULL,
    candidate_type VARCHAR(64) NOT NULL,
    state VARCHAR(64) NOT NULL,
    markdown_object_ref_id BIGINT,
    reviewer_id BIGINT,
    review_note TEXT,
    status INTEGER NOT NULL,
    created_at TIMESTAMP NOT NULL,
    updated_at TIMESTAMP NOT NULL,
    version BIGINT NOT NULL DEFAULT 0,
    PRIMARY KEY (id)
);

CREATE UNIQUE INDEX IF NOT EXISTS uk_kb_okf_candidate_uuid
    ON kb_okf_candidate (tenant_id, uuid);

CREATE INDEX IF NOT EXISTS idx_kb_okf_candidate_space_state
    ON kb_okf_candidate (tenant_id, space_id, state, updated_at);
