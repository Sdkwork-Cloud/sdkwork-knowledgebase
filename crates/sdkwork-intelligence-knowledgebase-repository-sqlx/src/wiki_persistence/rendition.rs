use async_trait::async_trait;
use sdkwork_intelligence_knowledgebase_service::ports::knowledge_wiki_persistence::{
    ClaimWikiRenditionsRequest, CompleteWikiRenditionRequest, UpsertWikiRenditionRequest,
    WikiPersistenceError, WikiPersistenceScope, WikiRenditionStore, WikiSourceRendition,
};
use sdkwork_utils_rust::uuid;
use sqlx::{any::AnyRow, Row};

use super::{
    claim_limit, from_i32, from_i64, lease_times, new_lease_token, optional_u64, parse_enum,
    require_id, require_sha256, require_text, row_error, sql_error, to_i64, validate_scope,
    SqlxWikiPersistenceStore,
};

const RENDITION_COLUMNS: &str = r#"
    id, uuid, tenant_id, organization_id, site_publication_id,
    source_file_projection_id, drive_version_uuid, source_content_sha256,
    processor_id, processor_version, policy_version, rendition_kind,
    rendition_key_sha256, rendition_state, rendition_drive_space_uuid,
    rendition_drive_node_uuid, rendition_drive_version_uuid, content_sha256,
    media_type, size_bytes, processing_attempt_count, processing_lease_token,
    processing_fence, version
"#;

#[async_trait]
impl WikiRenditionStore for SqlxWikiPersistenceStore {
    async fn upsert_rendition(
        &self,
        request: UpsertWikiRenditionRequest,
    ) -> Result<WikiSourceRendition, WikiPersistenceError> {
        validate_scope(request.scope)?;
        require_id("site_publication_id", request.site_publication_id)?;
        require_id(
            "source_file_projection_id",
            request.source_file_projection_id,
        )?;
        require_id("actor_id", request.actor_id)?;
        require_text("drive_version_uuid", &request.drive_version_uuid, 64)?;
        require_sha256("source_content_sha256", &request.source_content_sha256)?;
        require_text("processor_id", &request.processor_id, 128)?;
        require_text("processor_version", &request.processor_version, 64)?;
        require_text("policy_version", &request.policy_version, 64)?;
        require_sha256("rendition_key_sha256", &request.rendition_key_sha256)?;

        let query = format!(
            "SELECT {RENDITION_COLUMNS} FROM kb_source_file_rendition WHERE tenant_id = $1 AND organization_id = $2 AND source_file_projection_id = $3 AND rendition_key_sha256 = $4 AND status = 1",
        );
        if let Some(row) = sqlx::query(&query)
            .bind(to_i64("tenant_id", request.scope.tenant_id)?)
            .bind(to_i64("organization_id", request.scope.organization_id)?)
            .bind(require_id(
                "source_file_projection_id",
                request.source_file_projection_id,
            )?)
            .bind(request.rendition_key_sha256.as_str())
            .fetch_optional(&self.pool)
            .await
            .map_err(sql_error)?
        {
            let rendition = rendition_from_row(&row)?;
            if rendition.site_publication_id != request.site_publication_id
                || rendition.drive_version_uuid != request.drive_version_uuid.trim()
                || rendition.source_content_sha256 != request.source_content_sha256
                || rendition.processor_id != request.processor_id.trim()
                || rendition.processor_version != request.processor_version.trim()
                || rendition.policy_version != request.policy_version.trim()
                || rendition.rendition_kind != request.rendition_kind
            {
                return Err(WikiPersistenceError::Conflict(
                    "rendition_key_sha256 was reused for different deterministic inputs"
                        .to_string(),
                ));
            }
            return Ok(rendition);
        }

        let projection_identity = sqlx::query(
            r#"
            SELECT site_publication_id, drive_version_uuid, content_sha256
            FROM kb_source_file_projection
            WHERE tenant_id = $1 AND organization_id = $2 AND id = $3 AND status = 1
            "#,
        )
        .bind(to_i64("tenant_id", request.scope.tenant_id)?)
        .bind(to_i64("organization_id", request.scope.organization_id)?)
        .bind(require_id(
            "source_file_projection_id",
            request.source_file_projection_id,
        )?)
        .fetch_optional(&self.pool)
        .await
        .map_err(sql_error)?
        .ok_or(WikiPersistenceError::NotFound {
            resource: "wiki_source_projection",
            id: request.source_file_projection_id,
        })?;
        let projection_publication_id = from_i64(
            "site_publication_id",
            projection_identity
                .try_get("site_publication_id")
                .map_err(row_error)?,
        )?;
        let projection_drive_version: String = projection_identity
            .try_get("drive_version_uuid")
            .map_err(row_error)?;
        let projection_content_sha256: String = projection_identity
            .try_get("content_sha256")
            .map_err(row_error)?;
        if projection_publication_id != request.site_publication_id
            || projection_drive_version != request.drive_version_uuid.trim()
            || projection_content_sha256 != request.source_content_sha256
        {
            return Err(WikiPersistenceError::Conflict(
                "rendition inputs do not match the current source projection".to_string(),
            ));
        }

        let now = super::now()?;
        let timestamp = self.dialect.sql_timestamp_expr("$15");
        let query = format!(
            r#"
            INSERT INTO kb_source_file_rendition (
                id, uuid, tenant_id, organization_id, site_publication_id,
                source_file_projection_id, drive_version_uuid, source_content_sha256,
                processor_id, processor_version, policy_version, rendition_kind,
                rendition_key_sha256, created_by, updated_by, created_at, updated_at
            ) VALUES (
                $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12,
                $13, $14, $14, {timestamp}, {timestamp}
            )
            RETURNING {RENDITION_COLUMNS}
            "#,
        );
        let row = sqlx::query(&query)
            .bind(self.next_id()?)
            .bind(uuid())
            .bind(to_i64("tenant_id", request.scope.tenant_id)?)
            .bind(to_i64("organization_id", request.scope.organization_id)?)
            .bind(require_id(
                "site_publication_id",
                request.site_publication_id,
            )?)
            .bind(require_id(
                "source_file_projection_id",
                request.source_file_projection_id,
            )?)
            .bind(request.drive_version_uuid.trim())
            .bind(request.source_content_sha256.as_str())
            .bind(request.processor_id.trim())
            .bind(request.processor_version.trim())
            .bind(request.policy_version.trim())
            .bind(request.rendition_kind.as_str())
            .bind(request.rendition_key_sha256.as_str())
            .bind(require_id("actor_id", request.actor_id)?)
            .bind(&now)
            .fetch_one(&self.pool)
            .await
            .map_err(sql_error)?;
        rendition_from_row(&row)
    }

    async fn claim_renditions(
        &self,
        request: ClaimWikiRenditionsRequest,
    ) -> Result<Vec<WikiSourceRendition>, WikiPersistenceError> {
        validate_scope(request.scope)?;
        let owner = require_text("claim_owner", &request.claim_owner, 128)?;
        let limit = claim_limit(request.limit)?;
        let (now, lease_expires_at) = lease_times(request.lease_seconds)?;
        let now_expr = self.dialect.sql_timestamp_expr("$5");
        let mut transaction = self.pool.begin().await.map_err(sql_error)?;
        let candidate_query = format!(
            r#"
            SELECT id, version
            FROM kb_source_file_rendition
            WHERE tenant_id = $1 AND organization_id = $2 AND site_publication_id = $3
              AND id > COALESCE($4, 0) AND status = 1
              AND rendition_state IN ('PENDING', 'ERROR')
              AND (next_processing_at IS NULL OR next_processing_at <= {now_expr})
              AND (processing_lease_expires_at IS NULL OR processing_lease_expires_at <= {now_expr})
            ORDER BY id ASC
            LIMIT $6
            "#,
        );
        let candidates = sqlx::query(&candidate_query)
            .bind(to_i64("tenant_id", request.scope.tenant_id)?)
            .bind(to_i64("organization_id", request.scope.organization_id)?)
            .bind(require_id(
                "site_publication_id",
                request.site_publication_id,
            )?)
            .bind(
                request
                    .after_id
                    .map(|value| to_i64("after_id", value))
                    .transpose()?,
            )
            .bind(&now)
            .bind(limit)
            .fetch_all(&mut *transaction)
            .await
            .map_err(sql_error)?;

        let updated_at = self.dialect.sql_timestamp_expr("$7");
        let lease_expires_at_expr = self.dialect.sql_timestamp_expr("$8");
        let update_query = format!(
            r#"
            UPDATE kb_source_file_rendition
            SET rendition_state = 'PROCESSING',
                processing_attempt_count = processing_attempt_count + 1,
                processing_lease_owner = $5,
                processing_lease_token = $6,
                processing_lease_expires_at = {lease_expires_at_expr},
                processing_fence = processing_fence + 1,
                error_code = NULL,
                error_summary = NULL,
                updated_at = {updated_at},
                version = version + 1
            WHERE tenant_id = $1 AND organization_id = $2 AND id = $3
              AND version = $4 AND status = 1
            RETURNING {RENDITION_COLUMNS}
            "#,
        );
        let mut claimed = Vec::with_capacity(candidates.len());
        for candidate in candidates {
            let rendition_id: i64 = candidate.try_get("id").map_err(row_error)?;
            let expected_version: i64 = candidate.try_get("version").map_err(row_error)?;
            if let Some(row) = sqlx::query(&update_query)
                .bind(to_i64("tenant_id", request.scope.tenant_id)?)
                .bind(to_i64("organization_id", request.scope.organization_id)?)
                .bind(rendition_id)
                .bind(expected_version)
                .bind(owner)
                .bind(new_lease_token())
                .bind(&now)
                .bind(&lease_expires_at)
                .fetch_optional(&mut *transaction)
                .await
                .map_err(sql_error)?
            {
                claimed.push(rendition_from_row(&row)?);
            }
        }
        transaction.commit().await.map_err(sql_error)?;
        Ok(claimed)
    }

    async fn complete_rendition(
        &self,
        request: CompleteWikiRenditionRequest,
    ) -> Result<WikiSourceRendition, WikiPersistenceError> {
        validate_scope(request.scope)?;
        let lease_token = require_text("lease_token", &request.lease_token, 128)?;
        let rendition_drive_space_uuid = require_text(
            "rendition_drive_space_uuid",
            &request.rendition_drive_space_uuid,
            64,
        )?;
        let rendition_drive_node_uuid = require_text(
            "rendition_drive_node_uuid",
            &request.rendition_drive_node_uuid,
            64,
        )?;
        let rendition_drive_version_uuid = require_text(
            "rendition_drive_version_uuid",
            &request.rendition_drive_version_uuid,
            64,
        )?;
        require_sha256("content_sha256", &request.content_sha256)?;
        let media_type = require_text("media_type", &request.media_type, 255)?;
        let now = super::now()?;
        let timestamp = self.dialect.sql_timestamp_expr("$13");
        let query = format!(
            r#"
            UPDATE kb_source_file_rendition
            SET rendition_state = 'READY',
                rendition_drive_space_uuid = $6,
                rendition_drive_node_uuid = $7,
                rendition_drive_version_uuid = $8,
                content_sha256 = $9,
                media_type = $10,
                size_bytes = $11,
                processing_lease_owner = NULL,
                processing_lease_token = NULL,
                processing_lease_expires_at = NULL,
                error_code = NULL,
                error_summary = NULL,
                processed_at = {timestamp},
                updated_by = $12,
                updated_at = {timestamp},
                version = version + 1
            WHERE tenant_id = $1 AND organization_id = $2 AND id = $3
              AND processing_lease_token = $4 AND processing_fence = $5
              AND rendition_state = 'PROCESSING' AND status = 1
            RETURNING {RENDITION_COLUMNS}
            "#,
        );
        let row = sqlx::query(&query)
            .bind(to_i64("tenant_id", request.scope.tenant_id)?)
            .bind(to_i64("organization_id", request.scope.organization_id)?)
            .bind(require_id("rendition_id", request.rendition_id)?)
            .bind(lease_token)
            .bind(to_i64("processing_fence", request.processing_fence)?)
            .bind(rendition_drive_space_uuid)
            .bind(rendition_drive_node_uuid)
            .bind(rendition_drive_version_uuid)
            .bind(request.content_sha256.as_str())
            .bind(media_type)
            .bind(to_i64("size_bytes", request.size_bytes)?)
            .bind(require_id("actor_id", request.actor_id)?)
            .bind(&now)
            .fetch_optional(&self.pool)
            .await
            .map_err(sql_error)?
            .ok_or_else(|| {
                WikiPersistenceError::Conflict(format!(
                    "rendition {} processing lease is stale",
                    request.rendition_id
                ))
            })?;
        rendition_from_row(&row)
    }
}

fn rendition_from_row(row: &AnyRow) -> Result<WikiSourceRendition, WikiPersistenceError> {
    Ok(WikiSourceRendition {
        id: from_i64("id", row.try_get("id").map_err(row_error)?)?,
        uuid: row.try_get("uuid").map_err(row_error)?,
        scope: WikiPersistenceScope {
            tenant_id: from_i64("tenant_id", row.try_get("tenant_id").map_err(row_error)?)?,
            organization_id: from_i64(
                "organization_id",
                row.try_get("organization_id").map_err(row_error)?,
            )?,
        },
        site_publication_id: from_i64(
            "site_publication_id",
            row.try_get("site_publication_id").map_err(row_error)?,
        )?,
        source_file_projection_id: from_i64(
            "source_file_projection_id",
            row.try_get("source_file_projection_id")
                .map_err(row_error)?,
        )?,
        drive_version_uuid: row.try_get("drive_version_uuid").map_err(row_error)?,
        source_content_sha256: row.try_get("source_content_sha256").map_err(row_error)?,
        processor_id: row.try_get("processor_id").map_err(row_error)?,
        processor_version: row.try_get("processor_version").map_err(row_error)?,
        policy_version: row.try_get("policy_version").map_err(row_error)?,
        rendition_kind: parse_enum(
            "rendition_kind",
            row.try_get("rendition_kind").map_err(row_error)?,
        )?,
        rendition_key_sha256: row.try_get("rendition_key_sha256").map_err(row_error)?,
        rendition_state: parse_enum(
            "rendition_state",
            row.try_get("rendition_state").map_err(row_error)?,
        )?,
        rendition_drive_space_uuid: row
            .try_get("rendition_drive_space_uuid")
            .map_err(row_error)?,
        rendition_drive_node_uuid: row
            .try_get("rendition_drive_node_uuid")
            .map_err(row_error)?,
        rendition_drive_version_uuid: row
            .try_get("rendition_drive_version_uuid")
            .map_err(row_error)?,
        content_sha256: row.try_get("content_sha256").map_err(row_error)?,
        media_type: row.try_get("media_type").map_err(row_error)?,
        size_bytes: optional_u64(row, "size_bytes")?,
        processing_attempt_count: from_i32(
            "processing_attempt_count",
            row.try_get("processing_attempt_count").map_err(row_error)?,
        )?,
        processing_lease_token: row.try_get("processing_lease_token").map_err(row_error)?,
        processing_fence: from_i64(
            "processing_fence",
            row.try_get("processing_fence").map_err(row_error)?,
        )?,
        version: from_i64("version", row.try_get("version").map_err(row_error)?)?,
    })
}
