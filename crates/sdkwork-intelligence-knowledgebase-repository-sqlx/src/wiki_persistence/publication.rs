use async_trait::async_trait;
use sdkwork_intelligence_knowledgebase_service::ports::knowledge_wiki_persistence::{
    BindWikiSourceScopeRequest, ProvisionWikiPublicationRequest, WikiPersistenceError,
    WikiPersistenceScope, WikiPublication, WikiPublicationProvisioningResult, WikiPublicationStore,
};
use sdkwork_utils_rust::uuid;
use sqlx::{any::AnyRow, Row};

use super::{
    from_i64, now, parse_enum, require_id, require_text, row_error, sql_error, to_i64,
    validate_scope, SqlxWikiPersistenceStore,
};

pub(super) const PUBLICATION_COLUMNS: &str = r#"
    id, uuid, tenant_id, organization_id, space_id, drive_space_uuid,
    source_root_node_uuid, source_scope_uuid, wiki_status, title,
    homepage_source_path, publication_mode, default_visibility, update_policy,
    provider_generation, navigation_generation, search_generation,
    last_projected_drive_checkpoint, version
"#;

#[async_trait]
impl WikiPublicationStore for SqlxWikiPersistenceStore {
    async fn provision_publication(
        &self,
        request: ProvisionWikiPublicationRequest,
    ) -> Result<WikiPublicationProvisioningResult, WikiPersistenceError> {
        validate_scope(request.scope)?;
        let space_id = require_id("space_id", request.space_id)?;
        let actor_id = require_id("actor_id", request.actor_id)?;
        let drive_space_uuid = require_text("drive_space_uuid", &request.drive_space_uuid, 64)?;
        let title = require_text("title", &request.title, 256)?;

        if let Some(publication) = self
            .get_publication_for_space(request.scope, request.space_id)
            .await?
        {
            ensure_publication_identity(&publication, drive_space_uuid)?;
            return Ok(WikiPublicationProvisioningResult {
                publication,
                created: false,
            });
        }

        let drive_space_id: Option<String> = sqlx::query_scalar(
            r#"
            SELECT drive_space_id
            FROM kb_space
            WHERE tenant_id = $1 AND organization_id = $2 AND id = $3 AND status = 1
            "#,
        )
        .bind(to_i64("tenant_id", request.scope.tenant_id)?)
        .bind(to_i64("organization_id", request.scope.organization_id)?)
        .bind(space_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(sql_error)?
        .ok_or(WikiPersistenceError::NotFound {
            resource: "knowledge_space",
            id: request.space_id,
        })?;
        if drive_space_id.as_deref() != Some(drive_space_uuid) {
            return Err(WikiPersistenceError::Conflict(format!(
                "knowledge space {} is not bound to Drive Space {drive_space_uuid}",
                request.space_id
            )));
        }

        let id = self.next_id()?;
        let publication_uuid = uuid();
        let now = now()?;
        let created_at = self.dialect.sql_timestamp_expr("$9");
        let updated_at = self.dialect.sql_timestamp_expr("$10");
        let query = format!(
            r#"
            INSERT INTO kb_site_publication (
                id, uuid, tenant_id, organization_id, space_id, drive_space_uuid,
                title, created_by, updated_by, created_at, updated_at
            ) VALUES (
                $1, $2, $3, $4, $5, $6, $7, $8, $8, {created_at}, {updated_at}
            )
            RETURNING {PUBLICATION_COLUMNS}
            "#,
        );
        let insert_result = sqlx::query(&query)
            .bind(id)
            .bind(publication_uuid)
            .bind(to_i64("tenant_id", request.scope.tenant_id)?)
            .bind(to_i64("organization_id", request.scope.organization_id)?)
            .bind(space_id)
            .bind(drive_space_uuid)
            .bind(title)
            .bind(actor_id)
            .bind(&now)
            .bind(&now)
            .fetch_one(&self.pool)
            .await;

        match insert_result {
            Ok(row) => Ok(WikiPublicationProvisioningResult {
                publication: publication_from_row(&row)?,
                created: true,
            }),
            Err(error)
                if error
                    .as_database_error()
                    .is_some_and(|database_error| database_error.is_unique_violation()) =>
            {
                let publication = self
                    .get_publication_for_space(request.scope, request.space_id)
                    .await?
                    .ok_or_else(|| WikiPersistenceError::Conflict(error.to_string()))?;
                ensure_publication_identity(&publication, drive_space_uuid)?;
                Ok(WikiPublicationProvisioningResult {
                    publication,
                    created: false,
                })
            }
            Err(error) => Err(sql_error(error)),
        }
    }

    async fn get_publication(
        &self,
        scope: WikiPersistenceScope,
        site_publication_id: u64,
    ) -> Result<WikiPublication, WikiPersistenceError> {
        validate_scope(scope)?;
        let query = format!(
            "SELECT {PUBLICATION_COLUMNS} FROM kb_site_publication WHERE tenant_id = $1 AND organization_id = $2 AND id = $3 AND status = 1",
        );
        let row = sqlx::query(&query)
            .bind(to_i64("tenant_id", scope.tenant_id)?)
            .bind(to_i64("organization_id", scope.organization_id)?)
            .bind(require_id("site_publication_id", site_publication_id)?)
            .fetch_optional(&self.pool)
            .await
            .map_err(sql_error)?
            .ok_or(WikiPersistenceError::NotFound {
                resource: "wiki_publication",
                id: site_publication_id,
            })?;
        publication_from_row(&row)
    }

    async fn get_publication_for_space(
        &self,
        scope: WikiPersistenceScope,
        space_id: u64,
    ) -> Result<Option<WikiPublication>, WikiPersistenceError> {
        validate_scope(scope)?;
        let query = format!(
            "SELECT {PUBLICATION_COLUMNS} FROM kb_site_publication WHERE tenant_id = $1 AND organization_id = $2 AND space_id = $3 AND status = 1",
        );
        sqlx::query(&query)
            .bind(to_i64("tenant_id", scope.tenant_id)?)
            .bind(to_i64("organization_id", scope.organization_id)?)
            .bind(require_id("space_id", space_id)?)
            .fetch_optional(&self.pool)
            .await
            .map_err(sql_error)?
            .map(|row| publication_from_row(&row))
            .transpose()
    }

    async fn bind_source_scope(
        &self,
        request: BindWikiSourceScopeRequest,
    ) -> Result<WikiPublication, WikiPersistenceError> {
        validate_scope(request.scope)?;
        let root_uuid = require_text("source_root_node_uuid", &request.source_root_node_uuid, 64)?;
        let scope_uuid = require_text("source_scope_uuid", &request.source_scope_uuid, 64)?;
        let current = self
            .get_publication(request.scope, request.site_publication_id)
            .await?;
        if current.source_root_node_uuid.as_deref() == Some(root_uuid)
            && current.source_scope_uuid.as_deref() == Some(scope_uuid)
        {
            return Ok(current);
        }
        if current.source_root_node_uuid.is_some() || current.source_scope_uuid.is_some() {
            return Err(WikiPersistenceError::Conflict(format!(
                "Wiki publication {} is already bound to an immutable Drive raw scope",
                request.site_publication_id
            )));
        }
        if current.version != request.expected_version {
            return Err(WikiPersistenceError::StaleVersion {
                resource: "wiki_publication",
                id: request.site_publication_id,
                expected: request.expected_version,
            });
        }

        let now = now()?;
        let updated_at = self.dialect.sql_timestamp_expr("$8");
        let query = format!(
            r#"
            UPDATE kb_site_publication
            SET source_root_node_uuid = $4,
                source_scope_uuid = $5,
                wiki_status = 'VALIDATING',
                updated_by = $6,
                updated_at = {updated_at},
                version = version + 1
            WHERE tenant_id = $1 AND organization_id = $2 AND id = $3
              AND version = $7 AND status = 1
            RETURNING {PUBLICATION_COLUMNS}
            "#,
        );
        let row = sqlx::query(&query)
            .bind(to_i64("tenant_id", request.scope.tenant_id)?)
            .bind(to_i64("organization_id", request.scope.organization_id)?)
            .bind(require_id(
                "site_publication_id",
                request.site_publication_id,
            )?)
            .bind(root_uuid)
            .bind(scope_uuid)
            .bind(require_id("actor_id", request.actor_id)?)
            .bind(to_i64("expected_version", request.expected_version)?)
            .bind(&now)
            .fetch_optional(&self.pool)
            .await
            .map_err(sql_error)?
            .ok_or(WikiPersistenceError::StaleVersion {
                resource: "wiki_publication",
                id: request.site_publication_id,
                expected: request.expected_version,
            })?;
        publication_from_row(&row)
    }
}

fn ensure_publication_identity(
    publication: &WikiPublication,
    drive_space_uuid: &str,
) -> Result<(), WikiPersistenceError> {
    if publication.drive_space_uuid != drive_space_uuid {
        return Err(WikiPersistenceError::Conflict(format!(
            "knowledge space {} already owns a Wiki publication for Drive Space {}",
            publication.space_id, publication.drive_space_uuid
        )));
    }
    Ok(())
}

pub(super) fn publication_from_row(row: &AnyRow) -> Result<WikiPublication, WikiPersistenceError> {
    Ok(WikiPublication {
        id: from_i64("id", row.try_get("id").map_err(row_error)?)?,
        uuid: row.try_get("uuid").map_err(row_error)?,
        scope: WikiPersistenceScope {
            tenant_id: from_i64("tenant_id", row.try_get("tenant_id").map_err(row_error)?)?,
            organization_id: from_i64(
                "organization_id",
                row.try_get("organization_id").map_err(row_error)?,
            )?,
        },
        space_id: from_i64("space_id", row.try_get("space_id").map_err(row_error)?)?,
        drive_space_uuid: row.try_get("drive_space_uuid").map_err(row_error)?,
        source_root_node_uuid: row.try_get("source_root_node_uuid").map_err(row_error)?,
        source_scope_uuid: row.try_get("source_scope_uuid").map_err(row_error)?,
        wiki_status: parse_enum(
            "wiki_status",
            row.try_get("wiki_status").map_err(row_error)?,
        )?,
        title: row.try_get("title").map_err(row_error)?,
        homepage_source_path: row.try_get("homepage_source_path").map_err(row_error)?,
        publication_mode: parse_enum(
            "publication_mode",
            row.try_get("publication_mode").map_err(row_error)?,
        )?,
        default_visibility: parse_enum(
            "default_visibility",
            row.try_get("default_visibility").map_err(row_error)?,
        )?,
        update_policy: parse_enum(
            "update_policy",
            row.try_get("update_policy").map_err(row_error)?,
        )?,
        provider_generation: from_i64(
            "provider_generation",
            row.try_get("provider_generation").map_err(row_error)?,
        )?,
        navigation_generation: from_i64(
            "navigation_generation",
            row.try_get("navigation_generation").map_err(row_error)?,
        )?,
        search_generation: from_i64(
            "search_generation",
            row.try_get("search_generation").map_err(row_error)?,
        )?,
        last_projected_drive_checkpoint: from_i64(
            "last_projected_drive_checkpoint",
            row.try_get("last_projected_drive_checkpoint")
                .map_err(row_error)?,
        )?,
        version: from_i64("version", row.try_get("version").map_err(row_error)?)?,
    })
}
