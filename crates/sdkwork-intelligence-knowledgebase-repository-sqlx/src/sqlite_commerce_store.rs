use async_trait::async_trait;
use sdkwork_intelligence_knowledgebase_service::ports::commerce_store::{
    map_catalog_item, CreateSiteDeploymentRecord, KnowledgeMarketStore, KnowledgeMarketStoreError,
    KnowledgeSiteDeploymentStore, KnowledgeSiteDeploymentStoreError, SiteDeploymentRecord,
};
use sdkwork_knowledgebase_contract::market::KnowledgeMarketCatalogList;
use sdkwork_utils_rust::is_blank;
use sqlx::{AnyPool, Row};
use std::sync::Arc;
use time::{format_description::well_known::Rfc3339, OffsetDateTime};

use crate::id::{default_knowledge_id_generator, next_i64_id, KnowledgeIdGenerator};

const ACTIVE_STATUS: i64 = 1;
const DELETED_STATUS: i64 = 0;

#[derive(Debug, Clone)]
pub struct SqliteCommerceStore {
    pool: AnyPool,
    id_generator: Arc<dyn KnowledgeIdGenerator>,
}

impl SqliteCommerceStore {
    pub fn new(pool: AnyPool) -> Self {
        Self {
            pool,
            id_generator: default_knowledge_id_generator(),
        }
    }

    async fn bootstrap_market_listings_from_spaces(
        &self,
        tenant_id: u64,
    ) -> Result<(), KnowledgeMarketStoreError> {
        let rows = sqlx::query(
            r#"
            SELECT
                s.id,
                s.name,
                s.description,
                COALESCE((
                    SELECT COUNT(*)
                    FROM kb_document d
                    WHERE d.tenant_id = s.tenant_id
                      AND d.space_id = s.id
                      AND d.status = 1
                ), 0) AS documents_count
            FROM kb_space s
            WHERE s.tenant_id = $1 AND s.status = 1
            ORDER BY s.updated_at DESC
            LIMIT 12
            "#,
        )
        .bind(tenant_id as i64)
        .fetch_all(&self.pool)
        .await
        .map_err(|error| KnowledgeMarketStoreError::Internal(error.to_string()))?;

        if rows.is_empty() {
            return Ok(());
        }

        let now = now_rfc3339()?;
        for row in rows {
            let space_id = row.get::<i64, _>("id");
            let title = row.get::<String, _>("name");
            let description = row
                .try_get::<String, _>("description")
                .ok()
                .filter(|value| !is_blank(Some(value.as_str())))
                .unwrap_or_else(|| format!("Shared knowledge space: {title}"));
            let documents_count = row.get::<i64, _>("documents_count").max(0);
            let listing_id = next_i64_id(&self.id_generator)
                .map_err(|error| KnowledgeMarketStoreError::Internal(error.to_string()))?;

            let already_listed = sqlx::query_scalar::<_, i64>(
                "SELECT id FROM kb_market_listing WHERE tenant_id = $1 AND space_id = $2 AND status = 1",
            )
            .bind(tenant_id as i64)
            .bind(space_id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|error| KnowledgeMarketStoreError::Internal(error.to_string()))?;
            if already_listed.is_some() {
                continue;
            }

            sqlx::query(
                r#"
                INSERT INTO kb_market_listing (
                    id, tenant_id, space_id, title, icon, description, author, tags_json,
                    provider, model_name, subscribers_count, documents_count,
                    status, created_at, updated_at, version
                ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16)
                "#,
            )
            .bind(listing_id)
            .bind(tenant_id as i64)
            .bind(space_id)
            .bind(&title)
            .bind("📘")
            .bind(&description)
            .bind("SDKWork")
            .bind(r#"["知识共享","团队协同"]"#)
            .bind("Google")
            .bind("gemini-3.5-flash")
            .bind(0_i64)
            .bind(documents_count)
            .bind(ACTIVE_STATUS)
            .bind(&now)
            .bind(&now)
            .bind(0_i64)
            .execute(&self.pool)
            .await
            .map_err(|error| KnowledgeMarketStoreError::Internal(error.to_string()))?;
        }

        Ok(())
    }
}

fn now_rfc3339() -> Result<String, KnowledgeMarketStoreError> {
    OffsetDateTime::now_utc()
        .format(&Rfc3339)
        .map_err(|error| KnowledgeMarketStoreError::Internal(error.to_string()))
}

#[async_trait]
impl KnowledgeMarketStore for SqliteCommerceStore {
    async fn list_catalog(
        &self,
        tenant_id: u64,
        subscriber_actor_id: Option<u64>,
    ) -> Result<KnowledgeMarketCatalogList, KnowledgeMarketStoreError> {
        let rows = sqlx::query(
            r#"
            SELECT
                l.id, l.title, l.icon, l.description, l.author, l.tags_json,
                l.provider, l.model_name, l.subscribers_count, l.documents_count,
                CASE
                    WHEN $2 IS NULL THEN 0
                    WHEN EXISTS (
                        SELECT 1 FROM kb_market_subscription s
                        WHERE s.tenant_id = l.tenant_id
                          AND s.listing_id = l.id
                          AND s.subscriber_actor_id = $2
                          AND s.status = 1
                    ) THEN 1
                    ELSE 0
                END AS is_subscribed
            FROM kb_market_listing l
            WHERE l.tenant_id = $1 AND l.status = 1
            ORDER BY l.updated_at DESC
            LIMIT 200
            "#,
        )
        .bind(tenant_id as i64)
        .bind(subscriber_actor_id.map(|value| value as i64))
        .fetch_all(&self.pool)
        .await
        .map_err(|error| KnowledgeMarketStoreError::Internal(error.to_string()))?;

        let mut items = rows
            .into_iter()
            .map(|row| {
                map_catalog_item(
                    row.get::<i64, _>("id") as u64,
                    row.get("title"),
                    row.try_get("icon").ok(),
                    row.try_get("description").ok(),
                    row.try_get("author").ok(),
                    row.get("tags_json"),
                    row.try_get("provider").ok(),
                    row.try_get("model_name").ok(),
                    row.get::<i64, _>("subscribers_count") as u32,
                    row.get::<i64, _>("documents_count") as u32,
                    row.get::<i64, _>("is_subscribed") == 1,
                )
            })
            .collect::<Vec<_>>();

        if items.is_empty() {
            self.bootstrap_market_listings_from_spaces(tenant_id)
                .await?;
            let rows = sqlx::query(
                r#"
                SELECT
                    l.id, l.title, l.icon, l.description, l.author, l.tags_json,
                    l.provider, l.model_name, l.subscribers_count, l.documents_count,
                    CASE
                        WHEN $2 IS NULL THEN 0
                        WHEN EXISTS (
                            SELECT 1 FROM kb_market_subscription s
                            WHERE s.tenant_id = l.tenant_id
                              AND s.listing_id = l.id
                              AND s.subscriber_actor_id = $2
                              AND s.status = 1
                        ) THEN 1
                        ELSE 0
                    END AS is_subscribed
                FROM kb_market_listing l
                WHERE l.tenant_id = $1 AND l.status = 1
                ORDER BY l.updated_at DESC
                LIMIT 200
                "#,
            )
            .bind(tenant_id as i64)
            .bind(subscriber_actor_id.map(|value| value as i64))
            .fetch_all(&self.pool)
            .await
            .map_err(|error| KnowledgeMarketStoreError::Internal(error.to_string()))?;

            items = rows
                .into_iter()
                .map(|row| {
                    map_catalog_item(
                        row.get::<i64, _>("id") as u64,
                        row.get("title"),
                        row.try_get("icon").ok(),
                        row.try_get("description").ok(),
                        row.try_get("author").ok(),
                        row.get("tags_json"),
                        row.try_get("provider").ok(),
                        row.try_get("model_name").ok(),
                        row.get::<i64, _>("subscribers_count") as u32,
                        row.get::<i64, _>("documents_count") as u32,
                        row.get::<i64, _>("is_subscribed") == 1,
                    )
                })
                .collect();
        }

        Ok(KnowledgeMarketCatalogList { items })
    }

    async fn subscribe(
        &self,
        tenant_id: u64,
        subscriber_actor_id: u64,
        listing_id: u64,
    ) -> Result<(), KnowledgeMarketStoreError> {
        let listing_exists = sqlx::query_scalar::<_, i64>(
            "SELECT id FROM kb_market_listing WHERE tenant_id = $1 AND id = $2 AND status = 1",
        )
        .bind(tenant_id as i64)
        .bind(listing_id as i64)
        .fetch_optional(&self.pool)
        .await
        .map_err(|error| KnowledgeMarketStoreError::Internal(error.to_string()))?;
        if listing_exists.is_none() {
            return Err(KnowledgeMarketStoreError::NotFound);
        }

        let now = now_rfc3339()?;
        let id = next_i64_id(&self.id_generator)
            .map_err(|error| KnowledgeMarketStoreError::Internal(error.to_string()))?;
        sqlx::query(
            r#"
            INSERT INTO kb_market_subscription (
                id, tenant_id, subscriber_actor_id, listing_id, created_at, status
            ) VALUES ($1, $2, $3, $4, $5, $6)
            "#,
        )
        .bind(id)
        .bind(tenant_id as i64)
        .bind(subscriber_actor_id as i64)
        .bind(listing_id as i64)
        .bind(&now)
        .bind(ACTIVE_STATUS)
        .execute(&self.pool)
        .await
        .map_err(|error| {
            let message = error.to_string();
            if message.contains("UNIQUE") || message.contains("unique") {
                KnowledgeMarketStoreError::InvalidRequest(
                    "market listing is already subscribed".to_string(),
                )
            } else {
                KnowledgeMarketStoreError::Internal(message)
            }
        })?;

        sqlx::query(
            "UPDATE kb_market_listing SET subscribers_count = subscribers_count + 1, updated_at = $3 WHERE tenant_id = $1 AND id = $2",
        )
        .bind(tenant_id as i64)
        .bind(listing_id as i64)
        .bind(&now)
        .execute(&self.pool)
        .await
        .map_err(|error| KnowledgeMarketStoreError::Internal(error.to_string()))?;

        Ok(())
    }

    async fn unsubscribe(
        &self,
        tenant_id: u64,
        subscriber_actor_id: u64,
        listing_id: u64,
    ) -> Result<(), KnowledgeMarketStoreError> {
        let now = now_rfc3339()?;
        let result = sqlx::query(
            r#"
            UPDATE kb_market_subscription
            SET status = $4
            WHERE tenant_id = $1 AND subscriber_actor_id = $2 AND listing_id = $3 AND status = 1
            "#,
        )
        .bind(tenant_id as i64)
        .bind(subscriber_actor_id as i64)
        .bind(listing_id as i64)
        .bind(DELETED_STATUS)
        .execute(&self.pool)
        .await
        .map_err(|error| KnowledgeMarketStoreError::Internal(error.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(KnowledgeMarketStoreError::NotFound);
        }

        sqlx::query(
            "UPDATE kb_market_listing SET subscribers_count = CASE WHEN subscribers_count > 0 THEN subscribers_count - 1 ELSE 0 END, updated_at = $3 WHERE tenant_id = $1 AND id = $2",
        )
        .bind(tenant_id as i64)
        .bind(listing_id as i64)
        .bind(&now)
        .execute(&self.pool)
        .await
        .map_err(|error| KnowledgeMarketStoreError::Internal(error.to_string()))?;

        Ok(())
    }
}

#[async_trait]
impl KnowledgeSiteDeploymentStore for SqliteCommerceStore {
    async fn create_deployment(
        &self,
        record: CreateSiteDeploymentRecord,
    ) -> Result<SiteDeploymentRecord, KnowledgeSiteDeploymentStoreError> {
        let now = OffsetDateTime::now_utc()
            .format(&Rfc3339)
            .map_err(|error| KnowledgeSiteDeploymentStoreError::Internal(error.to_string()))?;
        let id = next_i64_id(&self.id_generator)
            .map_err(|error| KnowledgeSiteDeploymentStoreError::Internal(error.to_string()))?;

        sqlx::query(
            r#"
            INSERT INTO kb_site_deployment (
                id, tenant_id, space_id, platform, site_name, custom_domain,
                site_logo_data_url, deployed_url, preview_object_key,
                status, created_at, updated_at, version
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
            "#,
        )
        .bind(id)
        .bind(record.tenant_id as i64)
        .bind(record.space_id as i64)
        .bind(&record.platform)
        .bind(&record.site_name)
        .bind(&record.custom_domain)
        .bind(&record.site_logo_data_url)
        .bind(&record.deployed_url)
        .bind(&record.preview_object_key)
        .bind(ACTIVE_STATUS)
        .bind(&now)
        .bind(&now)
        .bind(0_i64)
        .execute(&self.pool)
        .await
        .map_err(|error| KnowledgeSiteDeploymentStoreError::Internal(error.to_string()))?;

        Ok(SiteDeploymentRecord {
            id: id as u64,
            tenant_id: record.tenant_id,
            space_id: record.space_id,
            platform: record.platform,
            site_name: record.site_name,
            custom_domain: record.custom_domain,
            deployed_url: record.deployed_url,
            preview_object_key: record.preview_object_key,
        })
    }

    async fn get_deployment(
        &self,
        tenant_id: u64,
        deployment_id: u64,
    ) -> Result<SiteDeploymentRecord, KnowledgeSiteDeploymentStoreError> {
        let row = sqlx::query(
            r#"
            SELECT id, tenant_id, space_id, platform, site_name, custom_domain,
                   deployed_url, preview_object_key
            FROM kb_site_deployment
            WHERE tenant_id = $1 AND id = $2 AND status = 1
            "#,
        )
        .bind(tenant_id as i64)
        .bind(deployment_id as i64)
        .fetch_optional(&self.pool)
        .await
        .map_err(|error| KnowledgeSiteDeploymentStoreError::Internal(error.to_string()))?
        .ok_or(KnowledgeSiteDeploymentStoreError::NotFound)?;

        Ok(SiteDeploymentRecord {
            id: row.get::<i64, _>("id") as u64,
            tenant_id: row.get::<i64, _>("tenant_id") as u64,
            space_id: row.get::<i64, _>("space_id") as u64,
            platform: row.get("platform"),
            site_name: row.try_get("site_name").ok(),
            custom_domain: row.try_get("custom_domain").ok(),
            deployed_url: row.get("deployed_url"),
            preview_object_key: row.get("preview_object_key"),
        })
    }
}
