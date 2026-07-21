use std::{str::FromStr, sync::Arc};

use async_trait::async_trait;
use sdkwork_database_config::DatabaseEngine;
use sdkwork_intelligence_knowledgebase_service::ports::knowledge_site_store::{
    CompleteKnowledgeSiteReleaseRecord, CreateKnowledgeSiteHostBindingRecord,
    CreateKnowledgeSiteReleaseRecord, KnowledgeSiteStore, KnowledgeSiteStoreError,
    ResolvedPublicKnowledgeSite, UpsertKnowledgeSiteRecord,
};
use sdkwork_knowledgebase_contract::{
    KnowledgeSite, KnowledgeSiteHostBinding, KnowledgeSiteHostBindingType, KnowledgeSiteRelease,
};
use sdkwork_utils_rust::is_blank;
use sqlx::{any::AnyRow, AnyPool, Row};
use uuid::Uuid;

use crate::{
    db::sql_timestamp::{utc_sql_timestamp_text, SqlTimestampDialect},
    id::{default_knowledge_id_generator, next_i64_id, KnowledgeIdGenerator},
};

const ACTIVE_STATUS: i64 = 1;
const DELETED_STATUS: i64 = 0;
const MAX_PAGE_SIZE: u32 = 200;

#[derive(Debug, Clone)]
pub struct SqlxKnowledgeSiteStore {
    pool: AnyPool,
    tenant_id: u64,
    organization_id: u64,
    id_generator: Arc<dyn KnowledgeIdGenerator>,
    timestamp_dialect: SqlTimestampDialect,
}

impl SqlxKnowledgeSiteStore {
    pub fn new(pool: AnyPool, tenant_id: u64, organization_id: u64) -> Self {
        Self {
            pool,
            tenant_id,
            organization_id,
            id_generator: default_knowledge_id_generator(),
            timestamp_dialect: SqlTimestampDialect::default(),
        }
    }

    pub fn with_database_engine(mut self, database_engine: DatabaseEngine) -> Self {
        self.timestamp_dialect = SqlTimestampDialect::from_database_engine(database_engine);
        self
    }

    #[cfg(test)]
    pub fn with_id_generator(mut self, id_generator: Arc<dyn KnowledgeIdGenerator>) -> Self {
        self.id_generator = id_generator;
        self
    }

    async fn canonical_host_for_site(
        &self,
        site_id: u64,
    ) -> Result<Option<String>, KnowledgeSiteStoreError> {
        let row = sqlx::query(
            r#"
            SELECT normalized_host
            FROM kb_site_host_binding
            WHERE tenant_id = $1 AND organization_id = $2 AND site_id = $3
              AND canonical = 1 AND lifecycle_state = 'active' AND status = $4
            LIMIT 1
            "#,
        )
        .bind(to_i64("tenant_id", self.tenant_id)?)
        .bind(to_i64("organization_id", self.organization_id)?)
        .bind(to_i64("site_id", site_id)?)
        .bind(ACTIVE_STATUS)
        .fetch_optional(&self.pool)
        .await
        .map_err(sqlx_error)?;
        Ok(row.map(|row| row.get("normalized_host")))
    }

    async fn resolve_public_site_from_row(
        &self,
        row: AnyRow,
    ) -> Result<ResolvedPublicKnowledgeSite, KnowledgeSiteStoreError> {
        let site = site_from_prefixed_row(&row)?;
        let release = release_from_prefixed_row(&row)?;
        let canonical_host = self.canonical_host_for_site(site.id).await?;
        Ok(ResolvedPublicKnowledgeSite {
            site,
            release,
            canonical_host,
        })
    }
}

#[async_trait]
impl KnowledgeSiteStore for SqlxKnowledgeSiteStore {
    async fn upsert_site(
        &self,
        record: UpsertKnowledgeSiteRecord,
    ) -> Result<KnowledgeSite, KnowledgeSiteStoreError> {
        validate_site_record(&record)?;
        let tenant_id = to_i64("tenant_id", self.tenant_id)?;
        let organization_id = to_i64("organization_id", self.organization_id)?;
        let space_id = to_i64("space_id", record.space_id)?;
        let existing = sqlx::query(
            r#"
            SELECT id, version
            FROM kb_site
            WHERE tenant_id = $1 AND organization_id = $2 AND space_id = $3 AND status = $4
            "#,
        )
        .bind(tenant_id)
        .bind(organization_id)
        .bind(space_id)
        .bind(ACTIVE_STATUS)
        .fetch_optional(&self.pool)
        .await
        .map_err(sqlx_error)?;

        if let Some(existing) = existing {
            let id: i64 = existing.get("id");
            let version: i64 = existing.get("version");
            let expected_version = record.expected_version.ok_or_else(|| {
                KnowledgeSiteStoreError::InvalidRequest(
                    "expected_version is required when updating an existing site".to_string(),
                )
            })?;
            if to_i64("expected_version", expected_version)? != version {
                return Err(KnowledgeSiteStoreError::VersionConflict);
            }
            let now = utc_sql_timestamp_text().map_err(KnowledgeSiteStoreError::Internal)?;
            let timestamp = self.timestamp_dialect.sql_timestamp_expr("$7");
            let query = format!(
                r#"
                UPDATE kb_site
                SET title = $1, visibility = $2, homepage_concept_id = $3,
                    theme_id = $4, publish_mode = $5, updated_at = {timestamp},
                    version = version + 1
                WHERE tenant_id = $6 AND id = $8 AND version = $9 AND status = $10
                "#
            );
            let result = sqlx::query(&query)
                .bind(record.title.trim())
                .bind(record.visibility.as_str())
                .bind(normalize_optional(record.homepage_concept_id))
                .bind(record.theme_id.trim())
                .bind(record.publish_mode.as_str())
                .bind(tenant_id)
                .bind(now)
                .bind(id)
                .bind(version)
                .bind(ACTIVE_STATUS)
                .execute(&self.pool)
                .await
                .map_err(sqlx_error)?;
            if result.rows_affected() != 1 {
                return Err(KnowledgeSiteStoreError::VersionConflict);
            }
            return self.get_site(to_u64("site id", id)?).await;
        }

        if record.expected_version.is_some() {
            return Err(KnowledgeSiteStoreError::NotFound);
        }
        let id = next_i64_id(&self.id_generator)
            .map_err(|error| KnowledgeSiteStoreError::Internal(error.to_string()))?;
        let uuid = Uuid::new_v4().to_string();
        let now = utc_sql_timestamp_text().map_err(KnowledgeSiteStoreError::Internal)?;
        let timestamp = self.timestamp_dialect.sql_timestamp_expr("$13");
        let query = format!(
            r#"
            INSERT INTO kb_site (
                id, uuid, tenant_id, organization_id, space_id, title, visibility,
                homepage_concept_id, theme_id, publish_mode, lifecycle_state, status,
                created_at, updated_at, version
            ) VALUES (
                $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, 'draft', $11,
                {timestamp}, {timestamp}, $12
            )
            "#
        );
        sqlx::query(&query)
            .bind(id)
            .bind(uuid)
            .bind(tenant_id)
            .bind(organization_id)
            .bind(space_id)
            .bind(record.title.trim())
            .bind(record.visibility.as_str())
            .bind(normalize_optional(record.homepage_concept_id))
            .bind(record.theme_id.trim())
            .bind(record.publish_mode.as_str())
            .bind(ACTIVE_STATUS)
            .bind(0_i64)
            .bind(now)
            .execute(&self.pool)
            .await
            .map_err(sqlx_error)?;
        self.get_site(to_u64("site id", id)?).await
    }

    async fn get_site_by_space(
        &self,
        space_id: u64,
    ) -> Result<KnowledgeSite, KnowledgeSiteStoreError> {
        let row = sqlx::query(&site_select_sql(
            "tenant_id = $1 AND organization_id = $2 AND space_id = $3 AND status = $4",
        ))
        .bind(to_i64("tenant_id", self.tenant_id)?)
        .bind(to_i64("organization_id", self.organization_id)?)
        .bind(to_i64("space_id", space_id)?)
        .bind(ACTIVE_STATUS)
        .fetch_optional(&self.pool)
        .await
        .map_err(sqlx_error)?
        .ok_or(KnowledgeSiteStoreError::NotFound)?;
        site_from_row(&row)
    }

    async fn get_site(&self, site_id: u64) -> Result<KnowledgeSite, KnowledgeSiteStoreError> {
        let row = sqlx::query(&site_select_sql(
            "tenant_id = $1 AND organization_id = $2 AND id = $3 AND status = $4",
        ))
        .bind(to_i64("tenant_id", self.tenant_id)?)
        .bind(to_i64("organization_id", self.organization_id)?)
        .bind(to_i64("site_id", site_id)?)
        .bind(ACTIVE_STATUS)
        .fetch_optional(&self.pool)
        .await
        .map_err(sqlx_error)?
        .ok_or(KnowledgeSiteStoreError::NotFound)?;
        site_from_row(&row)
    }

    async fn create_release(
        &self,
        record: CreateKnowledgeSiteReleaseRecord,
    ) -> Result<KnowledgeSiteRelease, KnowledgeSiteStoreError> {
        validate_hash(&record.source_content_hash)?;
        self.get_site(record.site_id).await?;
        let tenant_id = to_i64("tenant_id", self.tenant_id)?;
        let site_id = to_i64("site_id", record.site_id)?;
        if let Some(row) = sqlx::query(&release_select_sql(
            "tenant_id = $1 AND site_id = $2 AND source_content_hash = $3 AND status = $4",
        ))
        .bind(tenant_id)
        .bind(site_id)
        .bind(record.source_content_hash.trim())
        .bind(ACTIVE_STATUS)
        .fetch_optional(&self.pool)
        .await
        .map_err(sqlx_error)?
        {
            return release_from_row(&row);
        }

        let id = next_i64_id(&self.id_generator)
            .map_err(|error| KnowledgeSiteStoreError::Internal(error.to_string()))?;
        let now = utc_sql_timestamp_text().map_err(KnowledgeSiteStoreError::Internal)?;
        let timestamp = self.timestamp_dialect.sql_timestamp_expr("$9");
        let query = format!(
            r#"
            INSERT INTO kb_site_release (
                id, uuid, tenant_id, organization_id, site_id, lifecycle_state,
                source_content_hash, previous_release_id, status, created_at, version
            ) VALUES ($1, $2, $3, $4, $5, 'building', $6, $7, $8, {timestamp}, 0)
            "#
        );
        sqlx::query(&query)
            .bind(id)
            .bind(Uuid::new_v4().to_string())
            .bind(tenant_id)
            .bind(to_i64("organization_id", self.organization_id)?)
            .bind(site_id)
            .bind(record.source_content_hash.trim())
            .bind(
                record
                    .previous_release_id
                    .map(|value| to_i64("previous_release_id", value))
                    .transpose()?,
            )
            .bind(ACTIVE_STATUS)
            .bind(now)
            .execute(&self.pool)
            .await
            .map_err(sqlx_error)?;
        self.get_release(to_u64("release id", id)?).await
    }

    async fn complete_release(
        &self,
        record: CompleteKnowledgeSiteReleaseRecord,
    ) -> Result<KnowledgeSiteRelease, KnowledgeSiteStoreError> {
        validate_complete_release(&record)?;
        let now = utc_sql_timestamp_text().map_err(KnowledgeSiteStoreError::Internal)?;
        let timestamp = self.timestamp_dialect.sql_timestamp_expr("$8");
        let query = format!(
            r#"
            UPDATE kb_site_release
            SET lifecycle_state = 'ready', manifest_drive_uri = $1,
                manifest_drive_space_id = $2, manifest_drive_node_id = $3,
                manifest_checksum_sha256_hex = $4, page_count = $5, asset_count = $6,
                completed_at = {timestamp}, version = version + 1
            WHERE tenant_id = $7 AND id = $9 AND lifecycle_state = 'building' AND status = $10
            "#
        );
        let result = sqlx::query(&query)
            .bind(record.manifest_drive_uri)
            .bind(record.manifest_drive_space_id)
            .bind(record.manifest_drive_node_id)
            .bind(record.manifest_checksum_sha256_hex)
            .bind(i64::from(record.page_count))
            .bind(i64::from(record.asset_count))
            .bind(to_i64("tenant_id", self.tenant_id)?)
            .bind(now)
            .bind(to_i64("release_id", record.release_id)?)
            .bind(ACTIVE_STATUS)
            .execute(&self.pool)
            .await
            .map_err(sqlx_error)?;
        if result.rows_affected() != 1 {
            return Err(KnowledgeSiteStoreError::Conflict(
                "release is not in building state".to_string(),
            ));
        }
        self.get_release(record.release_id).await
    }

    async fn fail_release(
        &self,
        release_id: u64,
        error_code: String,
    ) -> Result<KnowledgeSiteRelease, KnowledgeSiteStoreError> {
        if is_blank(Some(error_code.as_str())) || error_code.len() > 128 {
            return Err(KnowledgeSiteStoreError::InvalidRequest(
                "error_code must contain 1 through 128 characters".to_string(),
            ));
        }
        let now = utc_sql_timestamp_text().map_err(KnowledgeSiteStoreError::Internal)?;
        let timestamp = self.timestamp_dialect.sql_timestamp_expr("$3");
        let query = format!(
            r#"
            UPDATE kb_site_release
            SET lifecycle_state = 'failed', error_code = $1, completed_at = {timestamp},
                version = version + 1
            WHERE tenant_id = $2 AND id = $4 AND lifecycle_state = 'building' AND status = $5
            "#
        );
        let result = sqlx::query(&query)
            .bind(error_code.trim())
            .bind(to_i64("tenant_id", self.tenant_id)?)
            .bind(now)
            .bind(to_i64("release_id", release_id)?)
            .bind(ACTIVE_STATUS)
            .execute(&self.pool)
            .await
            .map_err(sqlx_error)?;
        if result.rows_affected() != 1 {
            return Err(KnowledgeSiteStoreError::Conflict(
                "release is not in building state".to_string(),
            ));
        }
        self.get_release(release_id).await
    }

    async fn get_release(
        &self,
        release_id: u64,
    ) -> Result<KnowledgeSiteRelease, KnowledgeSiteStoreError> {
        let row = sqlx::query(&release_select_sql(
            "tenant_id = $1 AND organization_id = $2 AND id = $3 AND status = $4",
        ))
        .bind(to_i64("tenant_id", self.tenant_id)?)
        .bind(to_i64("organization_id", self.organization_id)?)
        .bind(to_i64("release_id", release_id)?)
        .bind(ACTIVE_STATUS)
        .fetch_optional(&self.pool)
        .await
        .map_err(sqlx_error)?
        .ok_or(KnowledgeSiteStoreError::NotFound)?;
        release_from_row(&row)
    }

    async fn list_releases_page(
        &self,
        site_id: u64,
        cursor: Option<u64>,
        page_size: u32,
    ) -> Result<(Vec<KnowledgeSiteRelease>, Option<u64>, bool), KnowledgeSiteStoreError> {
        let page_size = validate_page_size(page_size)?;
        let fetch_limit = i64::from(page_size + 1);
        let rows = if let Some(cursor) = cursor {
            sqlx::query(&format!(
                "{} ORDER BY id DESC LIMIT $6",
                release_select_sql(
                    "tenant_id = $1 AND organization_id = $2 AND site_id = $3 AND id < $4 AND status = $5"
                )
            ))
            .bind(to_i64("tenant_id", self.tenant_id)?)
            .bind(to_i64("organization_id", self.organization_id)?)
            .bind(to_i64("site_id", site_id)?)
            .bind(to_i64("cursor", cursor)?)
            .bind(ACTIVE_STATUS)
            .bind(fetch_limit)
            .fetch_all(&self.pool)
            .await
            .map_err(sqlx_error)?
        } else {
            sqlx::query(&format!(
                "{} ORDER BY id DESC LIMIT $5",
                release_select_sql(
                    "tenant_id = $1 AND organization_id = $2 AND site_id = $3 AND status = $4"
                )
            ))
            .bind(to_i64("tenant_id", self.tenant_id)?)
            .bind(to_i64("organization_id", self.organization_id)?)
            .bind(to_i64("site_id", site_id)?)
            .bind(ACTIVE_STATUS)
            .bind(fetch_limit)
            .fetch_all(&self.pool)
            .await
            .map_err(sqlx_error)?
        };
        page_releases(rows, page_size)
    }

    async fn activate_release(
        &self,
        site_id: u64,
        release_id: u64,
        expected_site_version: u64,
    ) -> Result<KnowledgeSite, KnowledgeSiteStoreError> {
        let tenant_id = to_i64("tenant_id", self.tenant_id)?;
        let site_id_i64 = to_i64("site_id", site_id)?;
        let release_id_i64 = to_i64("release_id", release_id)?;
        let mut tx = self.pool.begin().await.map_err(sqlx_error)?;
        let release = sqlx::query(
            r#"
            SELECT id FROM kb_site_release
            WHERE tenant_id = $1 AND organization_id = $2 AND site_id = $3 AND id = $4
              AND lifecycle_state = 'ready' AND status = $5
            "#,
        )
        .bind(tenant_id)
        .bind(to_i64("organization_id", self.organization_id)?)
        .bind(site_id_i64)
        .bind(release_id_i64)
        .bind(ACTIVE_STATUS)
        .fetch_optional(&mut *tx)
        .await
        .map_err(sqlx_error)?;
        if release.is_none() {
            return Err(KnowledgeSiteStoreError::NotFound);
        }
        let now = utc_sql_timestamp_text().map_err(KnowledgeSiteStoreError::Internal)?;
        let timestamp = self.timestamp_dialect.sql_timestamp_expr("$5");
        let query = format!(
            r#"
            UPDATE kb_site
            SET current_release_id = $1, lifecycle_state = 'active',
                updated_at = {timestamp}, version = version + 1
            WHERE tenant_id = $2 AND organization_id = $3 AND id = $4
              AND version = $6 AND status = $7
            "#
        );
        let result = sqlx::query(&query)
            .bind(release_id_i64)
            .bind(tenant_id)
            .bind(to_i64("organization_id", self.organization_id)?)
            .bind(site_id_i64)
            .bind(now)
            .bind(to_i64("expected_site_version", expected_site_version)?)
            .bind(ACTIVE_STATUS)
            .execute(&mut *tx)
            .await
            .map_err(sqlx_error)?;
        if result.rows_affected() != 1 {
            return Err(KnowledgeSiteStoreError::VersionConflict);
        }
        tx.commit().await.map_err(sqlx_error)?;
        self.get_site(site_id).await
    }

    async fn create_host_binding(
        &self,
        record: CreateKnowledgeSiteHostBindingRecord,
    ) -> Result<KnowledgeSiteHostBinding, KnowledgeSiteStoreError> {
        validate_host(&record.normalized_host)?;
        let tenant_id = to_i64("tenant_id", self.tenant_id)?;
        let site_id = to_i64("site_id", record.site_id)?;
        let expected_version = to_i64("expected_site_version", record.expected_site_version)?;
        let mut tx = self.pool.begin().await.map_err(sqlx_error)?;
        let site = sqlx::query(
            "SELECT version FROM kb_site WHERE tenant_id = $1 AND organization_id = $2 AND id = $3 AND status = $4",
        )
        .bind(tenant_id)
        .bind(to_i64("organization_id", self.organization_id)?)
        .bind(site_id)
        .bind(ACTIVE_STATUS)
        .fetch_optional(&mut *tx)
        .await
        .map_err(sqlx_error)?
        .ok_or(KnowledgeSiteStoreError::NotFound)?;
        if site.get::<i64, _>("version") != expected_version {
            return Err(KnowledgeSiteStoreError::VersionConflict);
        }
        if record.canonical {
            sqlx::query(
                "UPDATE kb_site_host_binding SET canonical = 0, version = version + 1 WHERE tenant_id = $1 AND site_id = $2 AND canonical = 1 AND status = $3",
            )
            .bind(tenant_id)
            .bind(site_id)
            .bind(ACTIVE_STATUS)
            .execute(&mut *tx)
            .await
            .map_err(sqlx_error)?;
        }
        let id = next_i64_id(&self.id_generator)
            .map_err(|error| KnowledgeSiteStoreError::Internal(error.to_string()))?;
        let now = utc_sql_timestamp_text().map_err(KnowledgeSiteStoreError::Internal)?;
        let timestamp = self.timestamp_dialect.sql_timestamp_expr("$14");
        let query = format!(
            r#"
            INSERT INTO kb_site_host_binding (
                id, uuid, tenant_id, organization_id, site_id, binding_type, normalized_host,
                canonical, lifecycle_state, web_server_site_id, web_server_domain_id,
                web_server_deployment_id, status, created_at, updated_at, version
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13,
                      {timestamp}, {timestamp}, 0)
            "#
        );
        sqlx::query(&query)
            .bind(id)
            .bind(Uuid::new_v4().to_string())
            .bind(tenant_id)
            .bind(to_i64("organization_id", self.organization_id)?)
            .bind(site_id)
            .bind(record.binding_type.as_str())
            .bind(record.normalized_host.trim())
            .bind(i64::from(record.canonical))
            .bind(record.lifecycle_state.as_str())
            .bind(record.web_server_site_id)
            .bind(record.web_server_domain_id)
            .bind(record.web_server_deployment_id)
            .bind(ACTIVE_STATUS)
            .bind(now.clone())
            .execute(&mut *tx)
            .await
            .map_err(sqlx_error)?;
        let update_timestamp = self.timestamp_dialect.sql_timestamp_expr("$4");
        let update = format!(
            r#"
            UPDATE kb_site
            SET canonical_host_binding_id = CASE WHEN $1 = 1 THEN $2 ELSE canonical_host_binding_id END,
                updated_at = {update_timestamp}, version = version + 1
            WHERE tenant_id = $3 AND id = $5 AND version = $6 AND status = $7
            "#
        );
        let result = sqlx::query(&update)
            .bind(i64::from(record.canonical))
            .bind(id)
            .bind(tenant_id)
            .bind(now)
            .bind(site_id)
            .bind(expected_version)
            .bind(ACTIVE_STATUS)
            .execute(&mut *tx)
            .await
            .map_err(sqlx_error)?;
        if result.rows_affected() != 1 {
            return Err(KnowledgeSiteStoreError::VersionConflict);
        }
        tx.commit().await.map_err(sqlx_error)?;
        get_host_binding(&self.pool, self.tenant_id, self.organization_id, to_u64("binding id", id)?)
            .await
    }

    async fn list_host_bindings_page(
        &self,
        site_id: u64,
        cursor: Option<u64>,
        page_size: u32,
    ) -> Result<(Vec<KnowledgeSiteHostBinding>, Option<u64>, bool), KnowledgeSiteStoreError> {
        let page_size = validate_page_size(page_size)?;
        let fetch_limit = i64::from(page_size + 1);
        let rows = if let Some(cursor) = cursor {
            sqlx::query(&format!(
                "{} ORDER BY id ASC LIMIT $6",
                host_binding_select_sql(
                    "tenant_id = $1 AND organization_id = $2 AND site_id = $3 AND id > $4 AND status = $5"
                )
            ))
            .bind(to_i64("tenant_id", self.tenant_id)?)
            .bind(to_i64("organization_id", self.organization_id)?)
            .bind(to_i64("site_id", site_id)?)
            .bind(to_i64("cursor", cursor)?)
            .bind(ACTIVE_STATUS)
            .bind(fetch_limit)
            .fetch_all(&self.pool)
            .await
            .map_err(sqlx_error)?
        } else {
            sqlx::query(&format!(
                "{} ORDER BY id ASC LIMIT $5",
                host_binding_select_sql(
                    "tenant_id = $1 AND organization_id = $2 AND site_id = $3 AND status = $4"
                )
            ))
            .bind(to_i64("tenant_id", self.tenant_id)?)
            .bind(to_i64("organization_id", self.organization_id)?)
            .bind(to_i64("site_id", site_id)?)
            .bind(ACTIVE_STATUS)
            .bind(fetch_limit)
            .fetch_all(&self.pool)
            .await
            .map_err(sqlx_error)?
        };
        page_host_bindings(rows, page_size)
    }

    async fn delete_host_binding(
        &self,
        site_id: u64,
        binding_id: u64,
        expected_site_version: u64,
    ) -> Result<(), KnowledgeSiteStoreError> {
        let binding = get_host_binding(
            &self.pool,
            self.tenant_id,
            self.organization_id,
            binding_id,
        )
        .await?;
        if binding.site_id != site_id {
            return Err(KnowledgeSiteStoreError::NotFound);
        }
        if binding.binding_type == KnowledgeSiteHostBindingType::SystemId {
            return Err(KnowledgeSiteStoreError::InvalidRequest(
                "the system ID host binding cannot be deleted".to_string(),
            ));
        }
        let mut tx = self.pool.begin().await.map_err(sqlx_error)?;
        let result = sqlx::query(
            "UPDATE kb_site_host_binding SET status = $1, canonical = 0, version = version + 1 WHERE tenant_id = $2 AND organization_id = $3 AND site_id = $4 AND id = $5 AND status = $6",
        )
        .bind(DELETED_STATUS)
        .bind(to_i64("tenant_id", self.tenant_id)?)
        .bind(to_i64("organization_id", self.organization_id)?)
        .bind(to_i64("site_id", site_id)?)
        .bind(to_i64("binding_id", binding_id)?)
        .bind(ACTIVE_STATUS)
        .execute(&mut *tx)
        .await
        .map_err(sqlx_error)?;
        if result.rows_affected() != 1 {
            return Err(KnowledgeSiteStoreError::NotFound);
        }
        let now = utc_sql_timestamp_text().map_err(KnowledgeSiteStoreError::Internal)?;
        let timestamp = self.timestamp_dialect.sql_timestamp_expr("$3");
        let query = format!(
            r#"
            UPDATE kb_site
            SET canonical_host_binding_id = CASE WHEN canonical_host_binding_id = $1 THEN NULL ELSE canonical_host_binding_id END,
                updated_at = {timestamp}, version = version + 1
            WHERE tenant_id = $2 AND organization_id = $4 AND id = $5 AND version = $6 AND status = $7
            "#
        );
        let result = sqlx::query(&query)
            .bind(to_i64("binding_id", binding_id)?)
            .bind(to_i64("tenant_id", self.tenant_id)?)
            .bind(now)
            .bind(to_i64("organization_id", self.organization_id)?)
            .bind(to_i64("site_id", site_id)?)
            .bind(to_i64("expected_site_version", expected_site_version)?)
            .bind(ACTIVE_STATUS)
            .execute(&mut *tx)
            .await
            .map_err(sqlx_error)?;
        if result.rows_affected() != 1 {
            return Err(KnowledgeSiteStoreError::VersionConflict);
        }
        tx.commit().await.map_err(sqlx_error)?;
        Ok(())
    }

    async fn resolve_public_site_by_space(
        &self,
        space_id: u64,
    ) -> Result<ResolvedPublicKnowledgeSite, KnowledgeSiteStoreError> {
        let row = sqlx::query(&public_site_select_sql(
            "s.tenant_id = $1 AND s.space_id = $2",
        ))
        .bind(to_i64("tenant_id", self.tenant_id)?)
        .bind(to_i64("space_id", space_id)?)
        .fetch_optional(&self.pool)
        .await
        .map_err(sqlx_error)?
        .ok_or(KnowledgeSiteStoreError::NotFound)?;
        self.resolve_public_site_from_row(row).await
    }

    async fn resolve_public_site_by_host(
        &self,
        normalized_host: &str,
    ) -> Result<ResolvedPublicKnowledgeSite, KnowledgeSiteStoreError> {
        validate_host(normalized_host)?;
        let row = sqlx::query(&format!(
            r#"
            {}
              AND EXISTS (
                SELECT 1 FROM kb_site_host_binding h
                WHERE h.tenant_id = s.tenant_id AND h.site_id = s.id
                  AND h.normalized_host = $2 AND h.lifecycle_state = 'active' AND h.status = 1
              )
            "#,
            public_site_select_sql("s.tenant_id = $1")
        ))
        .bind(to_i64("tenant_id", self.tenant_id)?)
        .bind(normalized_host.trim())
        .fetch_optional(&self.pool)
        .await
        .map_err(sqlx_error)?
        .ok_or(KnowledgeSiteStoreError::NotFound)?;
        self.resolve_public_site_from_row(row).await
    }
}

fn site_select_sql(predicate: &str) -> String {
    format!(
        r#"
        SELECT id, uuid, tenant_id, organization_id, space_id, title, visibility,
               homepage_concept_id, theme_id, publish_mode, lifecycle_state,
               canonical_host_binding_id, current_release_id,
               CAST(created_at AS TEXT) AS created_at,
               CAST(updated_at AS TEXT) AS updated_at, version
        FROM kb_site WHERE {predicate}
        "#
    )
}

fn release_select_sql(predicate: &str) -> String {
    format!(
        r#"
        SELECT id, uuid, site_id, lifecycle_state, source_content_hash,
               manifest_drive_uri, manifest_drive_space_id, manifest_drive_node_id,
               manifest_checksum_sha256_hex, page_count, asset_count, previous_release_id,
               error_code, CAST(created_at AS TEXT) AS created_at,
               CAST(completed_at AS TEXT) AS completed_at, version
        FROM kb_site_release WHERE {predicate}
        "#
    )
}

fn host_binding_select_sql(predicate: &str) -> String {
    format!(
        r#"
        SELECT id, uuid, site_id, binding_type, normalized_host, canonical,
               lifecycle_state, web_server_site_id, web_server_domain_id,
               web_server_deployment_id, CAST(created_at AS TEXT) AS created_at,
               CAST(updated_at AS TEXT) AS updated_at, version
        FROM kb_site_host_binding WHERE {predicate}
        "#
    )
}

fn public_site_select_sql(predicate: &str) -> String {
    format!(
        r#"
        SELECT
            s.id AS site_id_value, s.uuid AS site_uuid, s.tenant_id AS site_tenant_id,
            s.organization_id AS site_organization_id, s.space_id AS site_space_id,
            s.title AS site_title, s.visibility AS site_visibility,
            s.homepage_concept_id AS site_homepage_concept_id, s.theme_id AS site_theme_id,
            s.publish_mode AS site_publish_mode, s.lifecycle_state AS site_lifecycle_state,
            s.canonical_host_binding_id AS site_canonical_host_binding_id,
            s.current_release_id AS site_current_release_id,
            CAST(s.created_at AS TEXT) AS site_created_at,
            CAST(s.updated_at AS TEXT) AS site_updated_at, s.version AS site_version,
            r.id AS release_id_value, r.uuid AS release_uuid, r.site_id AS release_site_id,
            r.lifecycle_state AS release_lifecycle_state,
            r.source_content_hash AS release_source_content_hash,
            r.manifest_drive_uri AS release_manifest_drive_uri,
            r.manifest_drive_space_id AS release_manifest_drive_space_id,
            r.manifest_drive_node_id AS release_manifest_drive_node_id,
            r.manifest_checksum_sha256_hex AS release_manifest_checksum_sha256_hex,
            r.page_count AS release_page_count, r.asset_count AS release_asset_count,
            r.previous_release_id AS release_previous_release_id,
            r.error_code AS release_error_code,
            CAST(r.created_at AS TEXT) AS release_created_at,
            CAST(r.completed_at AS TEXT) AS release_completed_at,
            r.version AS release_version
        FROM kb_site s
        INNER JOIN kb_site_release r
          ON r.tenant_id = s.tenant_id AND r.site_id = s.id
         AND r.id = s.current_release_id AND r.lifecycle_state = 'ready' AND r.status = 1
        WHERE {predicate}
          AND s.lifecycle_state = 'active' AND s.visibility IN ('public', 'unlisted')
          AND s.status = 1
        "#
    )
}

fn site_from_row(row: &AnyRow) -> Result<KnowledgeSite, KnowledgeSiteStoreError> {
    Ok(KnowledgeSite {
        id: to_u64("site id", row.get("id"))?,
        uuid: row.get("uuid"),
        tenant_id: to_u64("tenant id", row.get("tenant_id"))?,
        organization_id: to_u64("organization id", row.get("organization_id"))?,
        space_id: to_u64("space id", row.get("space_id"))?,
        title: row.get("title"),
        visibility: parse_enum("visibility", row.get::<String, _>("visibility"))?,
        homepage_concept_id: row.get("homepage_concept_id"),
        theme_id: row.get("theme_id"),
        publish_mode: parse_enum("publish_mode", row.get::<String, _>("publish_mode"))?,
        lifecycle_state: parse_enum(
            "lifecycle_state",
            row.get::<String, _>("lifecycle_state"),
        )?,
        canonical_host_binding_id: optional_u64(
            "canonical host binding id",
            row.get("canonical_host_binding_id"),
        )?,
        current_release_id: optional_u64(
            "current release id",
            row.get("current_release_id"),
        )?,
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
        version: to_u64("site version", row.get("version"))?,
    })
}

fn release_from_row(row: &AnyRow) -> Result<KnowledgeSiteRelease, KnowledgeSiteStoreError> {
    Ok(KnowledgeSiteRelease {
        id: to_u64("release id", row.get("id"))?,
        uuid: row.get("uuid"),
        site_id: to_u64("site id", row.get("site_id"))?,
        lifecycle_state: parse_enum(
            "release lifecycle_state",
            row.get::<String, _>("lifecycle_state"),
        )?,
        source_content_hash: row.get("source_content_hash"),
        manifest_drive_uri: row.get("manifest_drive_uri"),
        manifest_drive_space_id: row.get("manifest_drive_space_id"),
        manifest_drive_node_id: row.get("manifest_drive_node_id"),
        manifest_checksum_sha256_hex: row.get("manifest_checksum_sha256_hex"),
        page_count: to_u32("page count", row.get("page_count"))?,
        asset_count: to_u32("asset count", row.get("asset_count"))?,
        previous_release_id: optional_u64(
            "previous release id",
            row.get("previous_release_id"),
        )?,
        error_code: row.get("error_code"),
        created_at: row.get("created_at"),
        completed_at: row.get("completed_at"),
        version: to_u64("release version", row.get("version"))?,
    })
}

fn host_binding_from_row(
    row: &AnyRow,
) -> Result<KnowledgeSiteHostBinding, KnowledgeSiteStoreError> {
    Ok(KnowledgeSiteHostBinding {
        id: to_u64("host binding id", row.get("id"))?,
        uuid: row.get("uuid"),
        site_id: to_u64("site id", row.get("site_id"))?,
        binding_type: parse_enum("binding_type", row.get::<String, _>("binding_type"))?,
        normalized_host: row.get("normalized_host"),
        canonical: row.get::<i64, _>("canonical") == 1,
        lifecycle_state: parse_enum(
            "binding lifecycle_state",
            row.get::<String, _>("lifecycle_state"),
        )?,
        web_server_site_id: row.get("web_server_site_id"),
        web_server_domain_id: row.get("web_server_domain_id"),
        web_server_deployment_id: row.get("web_server_deployment_id"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
        version: to_u64("host binding version", row.get("version"))?,
    })
}

fn site_from_prefixed_row(row: &AnyRow) -> Result<KnowledgeSite, KnowledgeSiteStoreError> {
    Ok(KnowledgeSite {
        id: to_u64("site id", row.get("site_id_value"))?,
        uuid: row.get("site_uuid"),
        tenant_id: to_u64("tenant id", row.get("site_tenant_id"))?,
        organization_id: to_u64("organization id", row.get("site_organization_id"))?,
        space_id: to_u64("space id", row.get("site_space_id"))?,
        title: row.get("site_title"),
        visibility: parse_enum("visibility", row.get::<String, _>("site_visibility"))?,
        homepage_concept_id: row.get("site_homepage_concept_id"),
        theme_id: row.get("site_theme_id"),
        publish_mode: parse_enum(
            "publish mode",
            row.get::<String, _>("site_publish_mode"),
        )?,
        lifecycle_state: parse_enum(
            "site lifecycle state",
            row.get::<String, _>("site_lifecycle_state"),
        )?,
        canonical_host_binding_id: optional_u64(
            "canonical host binding id",
            row.get("site_canonical_host_binding_id"),
        )?,
        current_release_id: optional_u64(
            "current release id",
            row.get("site_current_release_id"),
        )?,
        created_at: row.get("site_created_at"),
        updated_at: row.get("site_updated_at"),
        version: to_u64("site version", row.get("site_version"))?,
    })
}

fn release_from_prefixed_row(
    row: &AnyRow,
) -> Result<KnowledgeSiteRelease, KnowledgeSiteStoreError> {
    Ok(KnowledgeSiteRelease {
        id: to_u64("release id", row.get("release_id_value"))?,
        uuid: row.get("release_uuid"),
        site_id: to_u64("site id", row.get("release_site_id"))?,
        lifecycle_state: parse_enum(
            "release lifecycle state",
            row.get::<String, _>("release_lifecycle_state"),
        )?,
        source_content_hash: row.get("release_source_content_hash"),
        manifest_drive_uri: row.get("release_manifest_drive_uri"),
        manifest_drive_space_id: row.get("release_manifest_drive_space_id"),
        manifest_drive_node_id: row.get("release_manifest_drive_node_id"),
        manifest_checksum_sha256_hex: row.get("release_manifest_checksum_sha256_hex"),
        page_count: to_u32("page count", row.get("release_page_count"))?,
        asset_count: to_u32("asset count", row.get("release_asset_count"))?,
        previous_release_id: optional_u64(
            "previous release id",
            row.get("release_previous_release_id"),
        )?,
        error_code: row.get("release_error_code"),
        created_at: row.get("release_created_at"),
        completed_at: row.get("release_completed_at"),
        version: to_u64("release version", row.get("release_version"))?,
    })
}

async fn get_host_binding(
    pool: &AnyPool,
    tenant_id: u64,
    organization_id: u64,
    binding_id: u64,
) -> Result<KnowledgeSiteHostBinding, KnowledgeSiteStoreError> {
    let row = sqlx::query(&host_binding_select_sql(
        "tenant_id = $1 AND organization_id = $2 AND id = $3 AND status = $4",
    ))
    .bind(to_i64("tenant_id", tenant_id)?)
    .bind(to_i64("organization_id", organization_id)?)
    .bind(to_i64("binding_id", binding_id)?)
    .bind(ACTIVE_STATUS)
    .fetch_optional(pool)
    .await
    .map_err(sqlx_error)?
    .ok_or(KnowledgeSiteStoreError::NotFound)?;
    host_binding_from_row(&row)
}

fn page_releases(
    rows: Vec<AnyRow>,
    page_size: u32,
) -> Result<(Vec<KnowledgeSiteRelease>, Option<u64>, bool), KnowledgeSiteStoreError> {
    let has_more = rows.len() > page_size as usize;
    let items = rows
        .iter()
        .take(page_size as usize)
        .map(release_from_row)
        .collect::<Result<Vec<_>, _>>()?;
    let next_cursor = has_more.then(|| items.last().map(|item| item.id)).flatten();
    Ok((items, next_cursor, has_more))
}

fn page_host_bindings(
    rows: Vec<AnyRow>,
    page_size: u32,
) -> Result<(Vec<KnowledgeSiteHostBinding>, Option<u64>, bool), KnowledgeSiteStoreError> {
    let has_more = rows.len() > page_size as usize;
    let items = rows
        .iter()
        .take(page_size as usize)
        .map(host_binding_from_row)
        .collect::<Result<Vec<_>, _>>()?;
    let next_cursor = has_more.then(|| items.last().map(|item| item.id)).flatten();
    Ok((items, next_cursor, has_more))
}

fn validate_site_record(record: &UpsertKnowledgeSiteRecord) -> Result<(), KnowledgeSiteStoreError> {
    if record.space_id == 0 {
        return invalid("space_id is required");
    }
    if is_blank(Some(record.title.as_str())) || record.title.trim().chars().count() > 256 {
        return invalid("title must contain 1 through 256 characters");
    }
    if is_blank(Some(record.theme_id.as_str())) || record.theme_id.trim().len() > 64 {
        return invalid("theme_id must contain 1 through 64 characters");
    }
    if record
        .homepage_concept_id
        .as_deref()
        .is_some_and(|value| is_blank(Some(value)) || value.len() > 512)
    {
        return invalid("homepage_concept_id must contain 1 through 512 characters when present");
    }
    Ok(())
}

fn validate_complete_release(
    record: &CompleteKnowledgeSiteReleaseRecord,
) -> Result<(), KnowledgeSiteStoreError> {
    if record.release_id == 0 {
        return invalid("release_id is required");
    }
    if !record.manifest_drive_uri.starts_with("drive://spaces/")
        || !record.manifest_drive_uri.contains("/nodes/")
    {
        return invalid("manifest_drive_uri must be a stable Drive node URI");
    }
    if is_blank(Some(record.manifest_drive_space_id.as_str()))
        || is_blank(Some(record.manifest_drive_node_id.as_str()))
    {
        return invalid("manifest Drive space and node IDs are required");
    }
    validate_hash(&record.manifest_checksum_sha256_hex)
}

fn validate_hash(value: &str) -> Result<(), KnowledgeSiteStoreError> {
    let value = value.trim();
    if value.len() != 64 || !value.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return invalid("SHA-256 values must contain exactly 64 hexadecimal characters");
    }
    Ok(())
}

fn validate_host(value: &str) -> Result<(), KnowledgeSiteStoreError> {
    let value = value.trim();
    if value.is_empty()
        || value.len() > 253
        || value != value.to_ascii_lowercase()
        || value.starts_with('.')
        || value.ends_with('.')
        || value.split('.').any(|label| {
            label.is_empty()
                || label.len() > 63
                || label.starts_with('-')
                || label.ends_with('-')
                || !label
                    .bytes()
                    .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
        })
    {
        return invalid("host must be a normalized lowercase DNS hostname");
    }
    Ok(())
}

fn validate_page_size(page_size: u32) -> Result<u32, KnowledgeSiteStoreError> {
    if page_size == 0 || page_size > MAX_PAGE_SIZE {
        return invalid("page_size must be between 1 and 200");
    }
    Ok(page_size)
}

fn parse_enum<T: FromStr>(field: &str, value: String) -> Result<T, KnowledgeSiteStoreError> {
    T::from_str(&value).map_err(|_| {
        KnowledgeSiteStoreError::Internal(format!("invalid {field} persisted value: {value}"))
    })
}

fn normalize_optional(value: Option<String>) -> Option<String> {
    value.and_then(|value| {
        let value = value.trim();
        (!value.is_empty()).then(|| value.to_string())
    })
}

fn optional_u64(field: &str, value: Option<i64>) -> Result<Option<u64>, KnowledgeSiteStoreError> {
    value.map(|value| to_u64(field, value)).transpose()
}

fn to_i64(field: &str, value: u64) -> Result<i64, KnowledgeSiteStoreError> {
    i64::try_from(value).map_err(|_| {
        KnowledgeSiteStoreError::InvalidRequest(format!("{field} exceeds signed int64 range"))
    })
}

fn to_u64(field: &str, value: i64) -> Result<u64, KnowledgeSiteStoreError> {
    u64::try_from(value).map_err(|_| {
        KnowledgeSiteStoreError::Internal(format!("{field} is negative in persistence"))
    })
}

fn to_u32(field: &str, value: i64) -> Result<u32, KnowledgeSiteStoreError> {
    u32::try_from(value).map_err(|_| {
        KnowledgeSiteStoreError::Internal(format!("{field} exceeds unsigned int32 range"))
    })
}

fn invalid<T>(detail: &str) -> Result<T, KnowledgeSiteStoreError> {
    Err(KnowledgeSiteStoreError::InvalidRequest(detail.to_string()))
}

fn sqlx_error(error: sqlx::Error) -> KnowledgeSiteStoreError {
    if let sqlx::Error::Database(database) = &error {
        if database.is_unique_violation() {
            return KnowledgeSiteStoreError::Conflict(
                "site or host binding already exists".to_string(),
            );
        }
    }
    KnowledgeSiteStoreError::Internal(error.to_string())
}
