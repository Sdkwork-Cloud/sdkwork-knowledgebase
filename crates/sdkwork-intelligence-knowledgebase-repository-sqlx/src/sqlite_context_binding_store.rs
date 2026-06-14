use async_trait::async_trait;
use sdkwork_intelligence_knowledgebase_service::ports::knowledge_context_binding_store::{
    KnowledgeContextBindingStore, KnowledgeContextBindingStoreError,
};
use sdkwork_knowledgebase_contract::context_binding::{
    CreateKnowledgeSpaceContextBindingRequest, KnowledgeAccessLevel,
    KnowledgeSpaceContextBindingList, KnowledgeContextBindingStatus, KnowledgeContextType,
    KnowledgeSpaceContextBinding, ListContextBoundSpacesRequest,
    ListKnowledgeSpaceContextBindingsRequest, UpdateKnowledgeSpaceContextBindingRequest,
};
use sqlx::{Row, SqlitePool};
use std::sync::Arc;
use time::{format_description::well_known::Rfc3339, OffsetDateTime};

use crate::id::{default_knowledge_id_generator, next_i64_id, KnowledgeIdGenerator};

const ACTIVE_STATUS: i64 = 1;
const DELETED_STATUS: i64 = 0;
const INITIAL_VERSION: i64 = 0;

#[derive(Debug, Clone)]
pub struct SqliteContextBindingStore {
    pool: SqlitePool,
    id_generator: Arc<dyn KnowledgeIdGenerator>,
}

impl SqliteContextBindingStore {
    pub fn new(pool: SqlitePool) -> Self {
        Self {
            pool,
            id_generator: default_knowledge_id_generator(),
        }
    }

    pub fn with_id_generator(
        pool: SqlitePool,
        id_generator: Arc<dyn KnowledgeIdGenerator>,
    ) -> Self {
        Self { pool, id_generator }
    }
}

#[async_trait]
impl KnowledgeContextBindingStore for SqliteContextBindingStore {
    async fn create_binding(
        &self,
        tenant_id: u64,
        created_by: &str,
        request: CreateKnowledgeSpaceContextBindingRequest,
    ) -> Result<KnowledgeSpaceContextBinding, KnowledgeContextBindingStoreError> {
        let tenant_i64 = cb_to_i64("tenant_id", tenant_id)?;
        let space_i64 = cb_to_i64("space_id", request.space_id)?;
        let id = next_i64_id(&self.id_generator).map_err(cb_id_error)?;
        let now = cb_now()?;
        let context_type_str = request.context_type.as_str();
        let access_level_str = request
            .access_level
            .unwrap_or(KnowledgeAccessLevel::Reader)
            .as_str();

        let row = sqlx::query(
            r#"
            INSERT INTO kb_space_context_binding (
                id, tenant_id, space_id, context_type, context_id,
                context_name, access_level, status, created_by,
                created_at, updated_at, version
            )
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            RETURNING id, tenant_id, space_id, context_type, context_id,
                      context_name, access_level, status, created_by,
                      created_at, updated_at
            "#,
        )
        .bind(id)
        .bind(tenant_i64)
        .bind(space_i64)
        .bind(context_type_str)
        .bind(&request.context_id)
        .bind(&request.context_name)
        .bind(access_level_str)
        .bind(ACTIVE_STATUS)
        .bind(created_by)
        .bind(&now)
        .bind(&now)
        .bind(INITIAL_VERSION)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| {
            let msg = e.to_string();
            if msg.contains("UNIQUE") || msg.contains("unique") {
                KnowledgeContextBindingStoreError::Conflict(format!(
                    "binding already exists for space {} context {}:{}",
                    request.space_id, context_type_str, request.context_id
                ))
            } else {
                KnowledgeContextBindingStoreError::Internal(msg)
            }
        })?;

        cb_from_row(&row)
    }

    async fn get_binding(
        &self,
        tenant_id: u64,
        binding_id: u64,
    ) -> Result<KnowledgeSpaceContextBinding, KnowledgeContextBindingStoreError> {
        let tenant_i64 = cb_to_i64("tenant_id", tenant_id)?;
        let binding_i64 = cb_to_i64("binding_id", binding_id)?;

        let row = sqlx::query(
            r#"
            SELECT id, tenant_id, space_id, context_type, context_id,
                   context_name, access_level, status, created_by,
                   created_at, updated_at
            FROM kb_space_context_binding
            WHERE tenant_id = ? AND id = ? AND status = ?
            "#,
        )
        .bind(tenant_i64)
        .bind(binding_i64)
        .bind(ACTIVE_STATUS)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| KnowledgeContextBindingStoreError::Internal(e.to_string()))?;

        match row {
            Some(row) => cb_from_row(&row),
            None => Err(KnowledgeContextBindingStoreError::NotFound(binding_id)),
        }
    }

    async fn update_binding(
        &self,
        tenant_id: u64,
        binding_id: u64,
        request: UpdateKnowledgeSpaceContextBindingRequest,
    ) -> Result<KnowledgeSpaceContextBinding, KnowledgeContextBindingStoreError> {
        let tenant_i64 = cb_to_i64("tenant_id", tenant_id)?;
        let binding_i64 = cb_to_i64("binding_id", binding_id)?;
        let now = cb_now()?;

        let row = sqlx::query(
            r#"
            UPDATE kb_space_context_binding
            SET context_name = COALESCE(?, context_name),
                access_level = COALESCE(?, access_level),
                updated_at = ?,
                version = version + 1
            WHERE tenant_id = ? AND id = ? AND status = ?
            RETURNING id, tenant_id, space_id, context_type, context_id,
                      context_name, access_level, status, created_by,
                      created_at, updated_at
            "#,
        )
        .bind(&request.context_name)
        .bind(request.access_level.map(|l| l.as_str()))
        .bind(&now)
        .bind(tenant_i64)
        .bind(binding_i64)
        .bind(ACTIVE_STATUS)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| KnowledgeContextBindingStoreError::Internal(e.to_string()))?;

        match row {
            Some(row) => cb_from_row(&row),
            None => Err(KnowledgeContextBindingStoreError::NotFound(binding_id)),
        }
    }

    async fn delete_binding(
        &self,
        tenant_id: u64,
        binding_id: u64,
    ) -> Result<(), KnowledgeContextBindingStoreError> {
        let tenant_i64 = cb_to_i64("tenant_id", tenant_id)?;
        let binding_i64 = cb_to_i64("binding_id", binding_id)?;
        let now = cb_now()?;

        let result = sqlx::query(
            r#"
            UPDATE kb_space_context_binding
            SET status = ?, updated_at = ?, version = version + 1
            WHERE tenant_id = ? AND id = ? AND status = ?
            "#,
        )
        .bind(DELETED_STATUS)
        .bind(&now)
        .bind(tenant_i64)
        .bind(binding_i64)
        .bind(ACTIVE_STATUS)
        .execute(&self.pool)
        .await
        .map_err(|e| KnowledgeContextBindingStoreError::Internal(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(KnowledgeContextBindingStoreError::NotFound(binding_id));
        }

        Ok(())
    }

    async fn list_space_bindings(
        &self,
        tenant_id: u64,
        request: ListKnowledgeSpaceContextBindingsRequest,
    ) -> Result<KnowledgeSpaceContextBindingList, KnowledgeContextBindingStoreError> {
        let tenant_i64 = cb_to_i64("tenant_id", tenant_id)?;
        let space_i64 = cb_to_i64("space_id", request.space_id)?;
        let page_size = request.page_size.unwrap_or(50).min(200) as i64;

        let rows = if let Some(ref ctx_type) = request.context_type {
            let ctx_str = ctx_type.as_str();
            sqlx::query(
                r#"
                SELECT id, tenant_id, space_id, context_type, context_id,
                       context_name, access_level, status, created_by,
                       created_at, updated_at
                FROM kb_space_context_binding
                WHERE tenant_id = ? AND space_id = ? AND context_type = ? AND status = ?
                ORDER BY created_at
                LIMIT ?
                "#,
            )
            .bind(tenant_i64)
            .bind(space_i64)
            .bind(ctx_str)
            .bind(ACTIVE_STATUS)
            .bind(page_size + 1)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| KnowledgeContextBindingStoreError::Internal(e.to_string()))?
        } else {
            sqlx::query(
                r#"
                SELECT id, tenant_id, space_id, context_type, context_id,
                       context_name, access_level, status, created_by,
                       created_at, updated_at
                FROM kb_space_context_binding
                WHERE tenant_id = ? AND space_id = ? AND status = ?
                ORDER BY created_at
                LIMIT ?
                "#,
            )
            .bind(tenant_i64)
            .bind(space_i64)
            .bind(ACTIVE_STATUS)
            .bind(page_size + 1)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| KnowledgeContextBindingStoreError::Internal(e.to_string()))?
        };

        let has_more = rows.len() > page_size as usize;
        let mut items = Vec::new();
        for row in rows.into_iter().take(page_size as usize) {
            items.push(cb_from_row(&row)?);
        }

        let next_cursor = if has_more {
            items.last().map(|b| b.id.to_string())
        } else {
            None
        };

        Ok(KnowledgeSpaceContextBindingList { items, next_cursor })
    }

    async fn list_context_bound_spaces(
        &self,
        tenant_id: u64,
        request: ListContextBoundSpacesRequest,
    ) -> Result<Vec<u64>, KnowledgeContextBindingStoreError> {
        let tenant_i64 = cb_to_i64("tenant_id", tenant_id)?;
        let ctx_str = request.context_type.as_str();

        let rows = sqlx::query(
            r#"
            SELECT space_id
            FROM kb_space_context_binding
            WHERE tenant_id = ? AND context_type = ? AND context_id = ? AND status = ?
            ORDER BY created_at
            "#,
        )
        .bind(tenant_i64)
        .bind(ctx_str)
        .bind(&request.context_id)
        .bind(ACTIVE_STATUS)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| KnowledgeContextBindingStoreError::Internal(e.to_string()))?;

        let mut space_ids = Vec::new();
        for row in rows {
            let space_id_i64: i64 = row.get("space_id");
            let space_id = cb_from_i64("space_id", space_id_i64)?;
            space_ids.push(space_id);
        }

        Ok(space_ids)
    }
}

fn cb_from_row(row: &sqlx::sqlite::SqliteRow) -> Result<KnowledgeSpaceContextBinding, KnowledgeContextBindingStoreError> {
    let id: i64 = row.get("id");
    let tenant_id: i64 = row.get("tenant_id");
    let space_id: i64 = row.get("space_id");
    let context_type_str: String = row.get("context_type");
    let context_id: String = row.get("context_id");
    let context_name: Option<String> = row.get("context_name");
    let access_level_str: String = row.get("access_level");
    let status: i64 = row.get("status");
    let created_by: String = row.get("created_by");
    let created_at: String = row.get("created_at");
    let updated_at: String = row.get("updated_at");

    let context_type = KnowledgeContextType::from_str(&context_type_str)
        .ok_or_else(|| {
            KnowledgeContextBindingStoreError::Internal(format!(
                "unknown context_type: {context_type_str}"
            ))
        })?;

    let access_level = KnowledgeAccessLevel::from_str(&access_level_str)
        .ok_or_else(|| {
            KnowledgeContextBindingStoreError::Internal(format!(
                "unknown access_level: {access_level_str}"
            ))
        })?;

    let binding_status = if status == ACTIVE_STATUS {
        KnowledgeContextBindingStatus::Active
    } else {
        KnowledgeContextBindingStatus::Deleted
    };

    Ok(KnowledgeSpaceContextBinding {
        id: cb_from_i64("id", id)?,
        tenant_id: cb_from_i64("tenant_id", tenant_id)?,
        space_id: cb_from_i64("space_id", space_id)?,
        context_type,
        context_id,
        context_name,
        access_level,
        status: binding_status,
        created_by,
        created_at,
        updated_at,
    })
}

fn cb_now() -> Result<String, KnowledgeContextBindingStoreError> {
    OffsetDateTime::now_utc()
        .format(&Rfc3339)
        .map_err(|e| KnowledgeContextBindingStoreError::Internal(e.to_string()))
}

fn cb_to_i64(field: &str, value: u64) -> Result<i64, KnowledgeContextBindingStoreError> {
    i64::try_from(value).map_err(|_| {
        KnowledgeContextBindingStoreError::InvalidRequest(format!("{field} is out of range"))
    })
}

fn cb_from_i64(field: &str, value: i64) -> Result<u64, KnowledgeContextBindingStoreError> {
    u64::try_from(value).map_err(|_| {
        KnowledgeContextBindingStoreError::Internal(format!("{field} is negative"))
    })
}

fn cb_id_error(error: crate::KnowledgeIdGeneratorError) -> KnowledgeContextBindingStoreError {
    KnowledgeContextBindingStoreError::Internal(error.to_string())
}
