use async_trait::async_trait;
use sdkwork_intelligence_knowledgebase_service::ports::knowledge_provider_binding_readiness_store::{
    KnowledgeEngineProviderBindingReadinessGap, KnowledgeEngineProviderBindingReadinessGapPage,
    KnowledgeEngineProviderBindingReadinessStore,
    KnowledgeEngineProviderBindingReadinessStoreError,
    ListKnowledgeEngineProviderBindingReadinessGapsRequest,
};
use sdkwork_intelligence_knowledgebase_service::ports::knowledge_provider_binding_store::KnowledgeEngineProviderScope;
use sdkwork_utils_rust::{
    base64url_decode, base64url_encode, DEFAULT_LIST_PAGE_SIZE, MAX_LIST_PAGE_SIZE,
};
use sqlx::{AnyPool, Row};

const CURSOR_KIND: &str = "knowledgebase-provider-binding-readiness";
const CURSOR_VERSION: &str = "v1";
const MAX_CURSOR_LENGTH: usize = 512;

#[derive(Clone)]
pub struct SqlxKnowledgeEngineProviderBindingReadinessStore {
    pool: AnyPool,
}

impl SqlxKnowledgeEngineProviderBindingReadinessStore {
    pub fn new(pool: AnyPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl KnowledgeEngineProviderBindingReadinessStore
    for SqlxKnowledgeEngineProviderBindingReadinessStore
{
    async fn list_spaces_missing_active_binding(
        &self,
        scope: KnowledgeEngineProviderScope,
        request: ListKnowledgeEngineProviderBindingReadinessGapsRequest,
    ) -> Result<
        KnowledgeEngineProviderBindingReadinessGapPage,
        KnowledgeEngineProviderBindingReadinessStoreError,
    > {
        let tenant_id = to_i64("tenant_id", scope.tenant_id)?;
        let organization_id = to_i64("organization_id", scope.organization_id)?;
        if tenant_id <= 0 {
            return Err(invalid_request("tenant_id must be greater than zero"));
        }

        let page_size = request.page_size.unwrap_or(DEFAULT_LIST_PAGE_SIZE as u32);
        if page_size == 0 || page_size > MAX_LIST_PAGE_SIZE as u32 {
            return Err(invalid_request(format!(
                "page_size must be between 1 and {MAX_LIST_PAGE_SIZE}"
            )));
        }
        let cursor = decode_cursor(request.cursor.as_deref(), scope)?;
        let fetch_limit = i64::from(page_size) + 1;

        let rows = sqlx::query(
            r#"
            SELECT
                space.id,
                space.uuid,
                space.name,
                (
                    SELECT COUNT(*)
                    FROM kb_provider_binding AS binding
                    WHERE binding.tenant_id = space.tenant_id
                      AND binding.organization_id = space.organization_id
                      AND binding.space_id = space.id
                      AND binding.status = 1
                ) AS non_active_binding_count
            FROM kb_space AS space
            WHERE space.tenant_id = $1
              AND space.organization_id = $2
              AND space.knowledge_mode = 'external'
              AND space.status = 1
              AND ($3 IS NULL OR space.id < $3)
              AND NOT EXISTS (
                  SELECT 1
                  FROM kb_provider_binding AS active_binding
                  WHERE active_binding.tenant_id = space.tenant_id
                    AND active_binding.organization_id = space.organization_id
                    AND active_binding.space_id = space.id
                    AND active_binding.lifecycle_state = 'active'
                    AND active_binding.status = 1
              )
            ORDER BY space.id DESC
            LIMIT $4
            "#,
        )
        .bind(tenant_id)
        .bind(organization_id)
        .bind(cursor)
        .bind(fetch_limit)
        .fetch_all(&self.pool)
        .await
        .map_err(|_| KnowledgeEngineProviderBindingReadinessStoreError::QueryFailed)?;

        let has_more = rows.len() > page_size as usize;
        let items = rows
            .iter()
            .take(page_size as usize)
            .map(readiness_gap_from_row)
            .collect::<Result<Vec<_>, _>>()?;
        let next_cursor = if has_more {
            items
                .last()
                .map(|item| encode_cursor(scope, item.space_id))
                .transpose()?
        } else {
            None
        };

        Ok(KnowledgeEngineProviderBindingReadinessGapPage { items, next_cursor })
    }
}

fn readiness_gap_from_row(
    row: &sqlx::any::AnyRow,
) -> Result<
    KnowledgeEngineProviderBindingReadinessGap,
    KnowledgeEngineProviderBindingReadinessStoreError,
> {
    let space_id = row
        .try_get::<i64, _>("id")
        .map_err(|_| KnowledgeEngineProviderBindingReadinessStoreError::QueryFailed)?;
    let non_active_binding_count = row
        .try_get::<i64, _>("non_active_binding_count")
        .map_err(|_| KnowledgeEngineProviderBindingReadinessStoreError::QueryFailed)?;
    Ok(KnowledgeEngineProviderBindingReadinessGap {
        space_id: from_i64("space_id", space_id)?,
        space_uuid: row
            .try_get("uuid")
            .map_err(|_| KnowledgeEngineProviderBindingReadinessStoreError::QueryFailed)?,
        space_name: row
            .try_get("name")
            .map_err(|_| KnowledgeEngineProviderBindingReadinessStoreError::QueryFailed)?,
        non_active_binding_count: from_i64("non_active_binding_count", non_active_binding_count)?,
    })
}

fn encode_cursor(
    scope: KnowledgeEngineProviderScope,
    space_id: u64,
) -> Result<String, KnowledgeEngineProviderBindingReadinessStoreError> {
    to_i64("space_id", space_id)?;
    let payload = format!(
        "{CURSOR_KIND}:{CURSOR_VERSION}:{}:{}:{space_id}",
        scope.tenant_id, scope.organization_id
    );
    Ok(base64url_encode(payload.as_bytes()))
}

fn decode_cursor(
    cursor: Option<&str>,
    scope: KnowledgeEngineProviderScope,
) -> Result<Option<i64>, KnowledgeEngineProviderBindingReadinessStoreError> {
    let Some(cursor) = cursor.map(str::trim).filter(|value| !value.is_empty()) else {
        return Ok(None);
    };
    if cursor.len() > MAX_CURSOR_LENGTH {
        return Err(invalid_request("cursor exceeds the maximum length"));
    }
    let bytes = base64url_decode(cursor).ok_or_else(|| invalid_request("cursor is malformed"))?;
    let payload =
        std::str::from_utf8(&bytes).map_err(|_| invalid_request("cursor is malformed"))?;
    let mut parts = payload.split(':');
    let kind = parts.next();
    let version = parts.next();
    let tenant_id = parts.next().and_then(|value| value.parse::<u64>().ok());
    let organization_id = parts.next().and_then(|value| value.parse::<u64>().ok());
    let space_id = parts.next().and_then(|value| value.parse::<u64>().ok());
    if kind != Some(CURSOR_KIND)
        || version != Some(CURSOR_VERSION)
        || tenant_id != Some(scope.tenant_id)
        || organization_id != Some(scope.organization_id)
        || parts.next().is_some()
    {
        return Err(invalid_request(
            "cursor is invalid for the requested tenant and organization",
        ));
    }
    let space_id = space_id.ok_or_else(|| invalid_request("cursor is malformed"))?;
    Ok(Some(to_i64("cursor space_id", space_id)?))
}

fn to_i64(
    field: &str,
    value: u64,
) -> Result<i64, KnowledgeEngineProviderBindingReadinessStoreError> {
    i64::try_from(value)
        .map_err(|_| invalid_request(format!("{field} exceeds the signed 64-bit range")))
}

fn from_i64(
    field: &str,
    value: i64,
) -> Result<u64, KnowledgeEngineProviderBindingReadinessStoreError> {
    u64::try_from(value).map_err(|_| invalid_request(format!("{field} must not be negative")))
}

fn invalid_request(
    message: impl Into<String>,
) -> KnowledgeEngineProviderBindingReadinessStoreError {
    KnowledgeEngineProviderBindingReadinessStoreError::InvalidRequest(message.into())
}
