use async_trait::async_trait;
use sdkwork_intelligence_knowledgebase_service::ports::knowledge_wiki_persistence::{
    AdvanceWikiReconciliationRequest, ClaimWikiReconciliationRequest,
    CompleteWikiReconciliationRequest, ProvisionWikiDriveCheckpointRequest, WikiDriveCheckpoint,
    WikiDriveCheckpointStore, WikiPersistenceError, WikiPersistenceScope,
};
use sdkwork_utils_rust::uuid;
use sqlx::{any::AnyRow, Row};

use super::{
    from_i64, lease_times, new_lease_token, optional_u64, parse_enum, require_id, require_text,
    row_error, sql_error, to_i64, validate_scope, SqlxWikiPersistenceStore,
};

pub(super) const CHECKPOINT_COLUMNS: &str = r#"
    id, uuid, tenant_id, organization_id, site_publication_id,
    drive_space_uuid, source_scope_uuid, last_sequence_no, last_event_id,
    stream_state, gap_from_sequence_no, gap_to_sequence_no,
    reconciliation_cursor, lease_token, fence_token, version
"#;

#[async_trait]
impl WikiDriveCheckpointStore for SqlxWikiPersistenceStore {
    async fn provision_checkpoint(
        &self,
        request: ProvisionWikiDriveCheckpointRequest,
    ) -> Result<WikiDriveCheckpoint, WikiPersistenceError> {
        validate_scope(request.scope)?;
        let publication_id = require_id("site_publication_id", request.site_publication_id)?;
        let drive_space_uuid = require_text("drive_space_uuid", &request.drive_space_uuid, 64)?;
        let source_scope_uuid = require_text("source_scope_uuid", &request.source_scope_uuid, 64)?;

        if let Some(existing) =
            find_checkpoint_for_publication(self, request.scope, request.site_publication_id)
                .await?
        {
            ensure_checkpoint_identity(&existing, drive_space_uuid, source_scope_uuid)?;
            return Ok(existing);
        }

        let publication_identity = sqlx::query(
            r#"
            SELECT drive_space_uuid, source_scope_uuid
            FROM kb_site_publication
            WHERE tenant_id = $1 AND organization_id = $2 AND id = $3 AND status = 1
            "#,
        )
        .bind(to_i64("tenant_id", request.scope.tenant_id)?)
        .bind(to_i64("organization_id", request.scope.organization_id)?)
        .bind(publication_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(sql_error)?
        .ok_or(WikiPersistenceError::NotFound {
            resource: "wiki_publication",
            id: request.site_publication_id,
        })?;
        let publication_drive_space: String = publication_identity
            .try_get("drive_space_uuid")
            .map_err(row_error)?;
        let publication_source_scope: Option<String> = publication_identity
            .try_get("source_scope_uuid")
            .map_err(row_error)?;
        if publication_drive_space != drive_space_uuid
            || publication_source_scope.as_deref() != Some(source_scope_uuid)
        {
            return Err(WikiPersistenceError::Conflict(
                "checkpoint identity must match the bound Wiki publication source scope"
                    .to_string(),
            ));
        }

        let id = self.next_id()?;
        let checkpoint_uuid = uuid();
        let actor_id = require_id("actor_id", request.actor_id)?;
        let now = super::now()?;
        let created_at = self.dialect.sql_timestamp_expr("$9");
        let updated_at = self.dialect.sql_timestamp_expr("$10");
        let query = format!(
            r#"
            INSERT INTO kb_drive_source_checkpoint (
                id, uuid, tenant_id, organization_id, site_publication_id,
                drive_space_uuid, source_scope_uuid, created_by, updated_by,
                created_at, updated_at
            ) VALUES (
                $1, $2, $3, $4, $5, $6, $7, $8, $8, {created_at}, {updated_at}
            )
            RETURNING {CHECKPOINT_COLUMNS}
            "#,
        );
        let insert_result = sqlx::query(&query)
            .bind(id)
            .bind(checkpoint_uuid)
            .bind(to_i64("tenant_id", request.scope.tenant_id)?)
            .bind(to_i64("organization_id", request.scope.organization_id)?)
            .bind(publication_id)
            .bind(drive_space_uuid)
            .bind(source_scope_uuid)
            .bind(actor_id)
            .bind(&now)
            .bind(&now)
            .fetch_one(&self.pool)
            .await;
        match insert_result {
            Ok(row) => checkpoint_from_row(&row),
            Err(error)
                if error
                    .as_database_error()
                    .is_some_and(|database_error| database_error.is_unique_violation()) =>
            {
                let existing = find_checkpoint_for_publication(
                    self,
                    request.scope,
                    request.site_publication_id,
                )
                .await?
                .ok_or_else(|| WikiPersistenceError::Conflict(error.to_string()))?;
                ensure_checkpoint_identity(&existing, drive_space_uuid, source_scope_uuid)?;
                Ok(existing)
            }
            Err(error) => Err(sql_error(error)),
        }
    }

    async fn get_checkpoint(
        &self,
        scope: WikiPersistenceScope,
        checkpoint_id: u64,
    ) -> Result<WikiDriveCheckpoint, WikiPersistenceError> {
        validate_scope(scope)?;
        let query = format!(
            "SELECT {CHECKPOINT_COLUMNS} FROM kb_drive_source_checkpoint WHERE tenant_id = $1 AND organization_id = $2 AND id = $3 AND status = 1",
        );
        let row = sqlx::query(&query)
            .bind(to_i64("tenant_id", scope.tenant_id)?)
            .bind(to_i64("organization_id", scope.organization_id)?)
            .bind(require_id("checkpoint_id", checkpoint_id)?)
            .fetch_optional(&self.pool)
            .await
            .map_err(sql_error)?
            .ok_or(WikiPersistenceError::NotFound {
                resource: "wiki_drive_checkpoint",
                id: checkpoint_id,
            })?;
        checkpoint_from_row(&row)
    }

    async fn claim_reconciliation(
        &self,
        request: ClaimWikiReconciliationRequest,
    ) -> Result<WikiDriveCheckpoint, WikiPersistenceError> {
        validate_scope(request.scope)?;
        let owner = require_text("claim_owner", &request.claim_owner, 128)?;
        let (now, lease_expires_at) = lease_times(request.lease_seconds)?;
        let lease_token = new_lease_token();
        let reconciliation_started_at = self.dialect.sql_timestamp_expr("$8");
        let lease_expires_at_expr = self.dialect.sql_timestamp_expr("$9");
        let updated_at = self.dialect.sql_timestamp_expr("$10");
        let query = format!(
            r#"
            UPDATE kb_drive_source_checkpoint
            SET stream_state = 'RECONCILING',
                reconciliation_started_at = {reconciliation_started_at},
                lease_owner = $4,
                lease_token = $5,
                lease_expires_at = {lease_expires_at_expr},
                fence_token = fence_token + 1,
                updated_by = $6,
                updated_at = {updated_at},
                version = version + 1
            WHERE tenant_id = $1 AND organization_id = $2 AND id = $3
              AND version = $7 AND status = 1
              AND stream_state IN ('GAP_DETECTED', 'FAILED', 'RECONCILING')
              AND (lease_expires_at IS NULL OR lease_expires_at <= {reconciliation_started_at})
            RETURNING {CHECKPOINT_COLUMNS}
            "#,
        );
        let row = sqlx::query(&query)
            .bind(to_i64("tenant_id", request.scope.tenant_id)?)
            .bind(to_i64("organization_id", request.scope.organization_id)?)
            .bind(require_id("checkpoint_id", request.checkpoint_id)?)
            .bind(owner)
            .bind(lease_token)
            .bind(require_id("actor_id", request.actor_id)?)
            .bind(to_i64("expected_version", request.expected_version)?)
            .bind(&now)
            .bind(&lease_expires_at)
            .bind(&now)
            .fetch_optional(&self.pool)
            .await
            .map_err(sql_error)?
            .ok_or_else(|| {
                WikiPersistenceError::Conflict(format!(
                    "checkpoint {} is not claimable at version {}",
                    request.checkpoint_id, request.expected_version
                ))
            })?;
        checkpoint_from_row(&row)
    }

    async fn advance_reconciliation(
        &self,
        request: AdvanceWikiReconciliationRequest,
    ) -> Result<WikiDriveCheckpoint, WikiPersistenceError> {
        validate_scope(request.scope)?;
        let lease_token = require_text("lease_token", &request.lease_token, 128)?;
        let cursor = require_text(
            "reconciliation_cursor",
            &request.reconciliation_cursor,
            2_048,
        )?;
        let now = super::now()?;
        let timestamp = self.dialect.sql_timestamp_expr("$8");
        let query = format!(
            r#"
            UPDATE kb_drive_source_checkpoint
            SET reconciliation_cursor = $6,
                updated_by = $7,
                updated_at = {timestamp},
                version = version + 1
            WHERE tenant_id = $1 AND organization_id = $2 AND id = $3
              AND lease_token = $4 AND fence_token = $5
              AND stream_state = 'RECONCILING' AND status = 1
            RETURNING {CHECKPOINT_COLUMNS}
            "#,
        );
        let row = sqlx::query(&query)
            .bind(to_i64("tenant_id", request.scope.tenant_id)?)
            .bind(to_i64("organization_id", request.scope.organization_id)?)
            .bind(require_id("checkpoint_id", request.checkpoint_id)?)
            .bind(lease_token)
            .bind(to_i64("fence_token", request.fence_token)?)
            .bind(cursor)
            .bind(require_id("actor_id", request.actor_id)?)
            .bind(&now)
            .fetch_optional(&self.pool)
            .await
            .map_err(sql_error)?
            .ok_or_else(|| {
                WikiPersistenceError::Conflict(format!(
                    "checkpoint {} reconciliation lease is stale",
                    request.checkpoint_id
                ))
            })?;
        checkpoint_from_row(&row)
    }

    async fn complete_reconciliation(
        &self,
        request: CompleteWikiReconciliationRequest,
    ) -> Result<WikiDriveCheckpoint, WikiPersistenceError> {
        validate_scope(request.scope)?;
        let lease_token = require_text("lease_token", &request.lease_token, 128)?;
        if let Some(last_event_id) = request.last_event_id.as_deref() {
            require_text("last_event_id", last_event_id, 128)?;
        }
        let mut transaction = self.pool.begin().await.map_err(sql_error)?;
        let current_query = format!(
            "SELECT {CHECKPOINT_COLUMNS} FROM kb_drive_source_checkpoint WHERE tenant_id = $1 AND organization_id = $2 AND id = $3 AND lease_token = $4 AND fence_token = $5 AND stream_state = 'RECONCILING' AND status = 1",
        );
        let current_row = sqlx::query(&current_query)
            .bind(to_i64("tenant_id", request.scope.tenant_id)?)
            .bind(to_i64("organization_id", request.scope.organization_id)?)
            .bind(require_id("checkpoint_id", request.checkpoint_id)?)
            .bind(lease_token)
            .bind(to_i64("fence_token", request.fence_token)?)
            .fetch_optional(&mut *transaction)
            .await
            .map_err(sql_error)?
            .ok_or_else(|| {
                WikiPersistenceError::Conflict(format!(
                    "checkpoint {} reconciliation lease is stale",
                    request.checkpoint_id
                ))
            })?;
        let current = checkpoint_from_row(&current_row)?;
        if request.reconciled_sequence_no < current.last_sequence_no {
            return Err(WikiPersistenceError::Conflict(format!(
                "reconciled sequence {} is behind checkpoint sequence {}",
                request.reconciled_sequence_no, current.last_sequence_no
            )));
        }

        let inbox_updated_at = self.dialect.sql_timestamp_expr("$5");
        let inbox_reconcile_query = format!(
            r#"
            UPDATE kb_drive_event_inbox
            SET processing_state = CASE
                    WHEN processing_state = 'APPLIED' THEN processing_state
                    ELSE 'IGNORED'
                END,
                lease_owner = NULL,
                lease_token = NULL,
                lease_expires_at = NULL,
                updated_at = {inbox_updated_at},
                version = version + 1
            WHERE tenant_id = $1 AND organization_id = $2 AND checkpoint_id = $3
              AND sequence_no <= $4 AND processing_state <> 'APPLIED'
            "#,
        );
        sqlx::query(&inbox_reconcile_query)
            .bind(to_i64("tenant_id", request.scope.tenant_id)?)
            .bind(to_i64("organization_id", request.scope.organization_id)?)
            .bind(require_id("checkpoint_id", request.checkpoint_id)?)
            .bind(to_i64(
                "reconciled_sequence_no",
                request.reconciled_sequence_no,
            )?)
            .bind(super::now()?)
            .execute(&mut *transaction)
            .await
            .map_err(sql_error)?;

        let now = super::now()?;
        let timestamp = self.dialect.sql_timestamp_expr("$9");
        let update_query = format!(
            r#"
            UPDATE kb_drive_source_checkpoint
            SET last_sequence_no = $6,
                last_event_id = $7,
                stream_state = 'HEALTHY',
                gap_from_sequence_no = NULL,
                gap_to_sequence_no = NULL,
                reconciliation_cursor = NULL,
                reconciliation_completed_at = {timestamp},
                lease_owner = NULL,
                lease_token = NULL,
                lease_expires_at = NULL,
                last_observed_at = {timestamp},
                last_error_code = NULL,
                last_error_summary = NULL,
                updated_by = $8,
                updated_at = {timestamp},
                version = version + 1
            WHERE tenant_id = $1 AND organization_id = $2 AND id = $3
              AND lease_token = $4 AND fence_token = $5
              AND stream_state = 'RECONCILING' AND status = 1
            RETURNING {CHECKPOINT_COLUMNS}
            "#,
        );
        let completed_row = sqlx::query(&update_query)
            .bind(to_i64("tenant_id", request.scope.tenant_id)?)
            .bind(to_i64("organization_id", request.scope.organization_id)?)
            .bind(require_id("checkpoint_id", request.checkpoint_id)?)
            .bind(lease_token)
            .bind(to_i64("fence_token", request.fence_token)?)
            .bind(to_i64(
                "reconciled_sequence_no",
                request.reconciled_sequence_no,
            )?)
            .bind(request.last_event_id.as_deref())
            .bind(require_id("actor_id", request.actor_id)?)
            .bind(&now)
            .fetch_one(&mut *transaction)
            .await
            .map_err(sql_error)?;
        let completed = checkpoint_from_row(&completed_row)?;

        let publication_time = self.dialect.sql_timestamp_expr("$6");
        let publication_update = format!(
            r#"
            UPDATE kb_site_publication
            SET last_projected_drive_checkpoint = $4,
                updated_by = $5,
                updated_at = {publication_time},
                version = version + 1
            WHERE tenant_id = $1 AND organization_id = $2 AND id = $3
              AND last_projected_drive_checkpoint < $4 AND status = 1
            "#,
        );
        sqlx::query(&publication_update)
            .bind(to_i64("tenant_id", request.scope.tenant_id)?)
            .bind(to_i64("organization_id", request.scope.organization_id)?)
            .bind(to_i64("site_publication_id", current.site_publication_id)?)
            .bind(to_i64(
                "reconciled_sequence_no",
                request.reconciled_sequence_no,
            )?)
            .bind(require_id("actor_id", request.actor_id)?)
            .bind(&now)
            .execute(&mut *transaction)
            .await
            .map_err(sql_error)?;

        transaction.commit().await.map_err(sql_error)?;
        Ok(completed)
    }
}

async fn find_checkpoint_for_publication(
    store: &SqlxWikiPersistenceStore,
    scope: WikiPersistenceScope,
    site_publication_id: u64,
) -> Result<Option<WikiDriveCheckpoint>, WikiPersistenceError> {
    let query = format!(
        "SELECT {CHECKPOINT_COLUMNS} FROM kb_drive_source_checkpoint WHERE tenant_id = $1 AND organization_id = $2 AND site_publication_id = $3 AND status = 1",
    );
    sqlx::query(&query)
        .bind(to_i64("tenant_id", scope.tenant_id)?)
        .bind(to_i64("organization_id", scope.organization_id)?)
        .bind(require_id("site_publication_id", site_publication_id)?)
        .fetch_optional(&store.pool)
        .await
        .map_err(sql_error)?
        .map(|row| checkpoint_from_row(&row))
        .transpose()
}

fn ensure_checkpoint_identity(
    checkpoint: &WikiDriveCheckpoint,
    drive_space_uuid: &str,
    source_scope_uuid: &str,
) -> Result<(), WikiPersistenceError> {
    if checkpoint.drive_space_uuid != drive_space_uuid
        || checkpoint.source_scope_uuid != source_scope_uuid
    {
        return Err(WikiPersistenceError::Conflict(format!(
            "Wiki publication {} already owns a checkpoint for a different Drive source scope",
            checkpoint.site_publication_id
        )));
    }
    Ok(())
}

pub(super) fn checkpoint_from_row(
    row: &AnyRow,
) -> Result<WikiDriveCheckpoint, WikiPersistenceError> {
    Ok(WikiDriveCheckpoint {
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
        drive_space_uuid: row.try_get("drive_space_uuid").map_err(row_error)?,
        source_scope_uuid: row.try_get("source_scope_uuid").map_err(row_error)?,
        last_sequence_no: from_i64(
            "last_sequence_no",
            row.try_get("last_sequence_no").map_err(row_error)?,
        )?,
        last_event_id: row.try_get("last_event_id").map_err(row_error)?,
        stream_state: parse_enum(
            "stream_state",
            row.try_get("stream_state").map_err(row_error)?,
        )?,
        gap_from_sequence_no: optional_u64(row, "gap_from_sequence_no")?,
        gap_to_sequence_no: optional_u64(row, "gap_to_sequence_no")?,
        reconciliation_cursor: row.try_get("reconciliation_cursor").map_err(row_error)?,
        lease_token: row.try_get("lease_token").map_err(row_error)?,
        fence_token: from_i64(
            "fence_token",
            row.try_get("fence_token").map_err(row_error)?,
        )?,
        version: from_i64("version", row.try_get("version").map_err(row_error)?)?,
    })
}
