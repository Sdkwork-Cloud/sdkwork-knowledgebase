use async_trait::async_trait;
use sdkwork_intelligence_knowledgebase_service::ports::knowledge_okf_concept_link_store::{
    KnowledgeOkfConceptLinkEdge, KnowledgeOkfConceptLinkStore, KnowledgeOkfConceptLinkStoreError,
    ReplaceKnowledgeOkfConceptLinksRecord,
};
use sqlx::AnyPool;
use sqlx::Row;
use std::collections::BTreeSet;
use std::sync::Arc;
use time::{format_description::well_known::Rfc3339, OffsetDateTime};
use uuid::Uuid;

use crate::id::{default_knowledge_id_generator, next_i64_id, KnowledgeIdGenerator};

/// Maximum inbound link targets scanned for orphan concept detection per space.
pub const MAX_OKF_ORPHAN_LINK_TARGETS: i64 = 5000;

const ACTIVE_STATUS: i64 = 1;
const INITIAL_VERSION: i64 = 0;

#[derive(Debug, Clone)]
pub struct SqliteKnowledgeOkfConceptLinkStore {
    pool: AnyPool,
    tenant_id: u64,
    id_generator: Arc<dyn KnowledgeIdGenerator>,
}

impl SqliteKnowledgeOkfConceptLinkStore {
    pub fn new(pool: AnyPool, tenant_id: u64) -> Self {
        Self::with_id_generator(pool, tenant_id, default_knowledge_id_generator())
    }

    pub fn with_id_generator(
        pool: AnyPool,
        tenant_id: u64,
        id_generator: Arc<dyn KnowledgeIdGenerator>,
    ) -> Self {
        Self {
            pool,
            tenant_id,
            id_generator,
        }
    }
}

#[async_trait]
impl KnowledgeOkfConceptLinkStore for SqliteKnowledgeOkfConceptLinkStore {
    async fn replace_outbound_links(
        &self,
        record: ReplaceKnowledgeOkfConceptLinksRecord,
    ) -> Result<(), KnowledgeOkfConceptLinkStoreError> {
        let tenant_id = to_i64("tenant_id", self.tenant_id)?;
        let space_id = to_i64("space_id", record.space_id)?;
        let now = now_rfc3339()?;

        let mut transaction = self
            .pool
            .begin()
            .await
            .map_err(|error| KnowledgeOkfConceptLinkStoreError::Internal(error.to_string()))?;

        sqlx::query(
            r#"
            UPDATE kb_okf_concept_link
            SET status = 0, updated_at = CAST($1 AS TIMESTAMP), version = version + 1
            WHERE tenant_id = $2 AND space_id = $3 AND from_concept_id = $4 AND status = $5
            "#,
        )
        .bind(&now)
        .bind(tenant_id)
        .bind(space_id)
        .bind(&record.from_concept_id)
        .bind(ACTIVE_STATUS)
        .execute(&mut *transaction)
        .await
        .map_err(|error| KnowledgeOkfConceptLinkStoreError::Internal(error.to_string()))?;

        for link in record.links {
            let id = next_i64_id(&self.id_generator).map_err(id_error)?;
            sqlx::query(
                r#"
                INSERT INTO kb_okf_concept_link (
                    id, uuid, tenant_id, space_id, from_concept_id, to_concept_id,
                    anchor_text, status, created_at, updated_at, version
                ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
                "#,
            )
            .bind(id)
            .bind(Uuid::new_v4().to_string())
            .bind(tenant_id)
            .bind(space_id)
            .bind(&record.from_concept_id)
            .bind(&link.to_concept_id)
            .bind(&link.anchor_text)
            .bind(ACTIVE_STATUS)
            .bind(&now)
            .bind(&now)
            .bind(INITIAL_VERSION)
            .execute(&mut *transaction)
            .await
            .map_err(|error| KnowledgeOkfConceptLinkStoreError::Internal(error.to_string()))?;
        }

        transaction
            .commit()
            .await
            .map_err(|error| KnowledgeOkfConceptLinkStoreError::Internal(error.to_string()))?;
        Ok(())
    }

    async fn list_inbound_concept_ids(
        &self,
        space_id: u64,
        to_concept_id: &str,
    ) -> Result<Vec<String>, KnowledgeOkfConceptLinkStoreError> {
        let tenant_id = to_i64("tenant_id", self.tenant_id)?;
        let space_id = to_i64("space_id", space_id)?;
        let rows = sqlx::query_scalar::<_, String>(
            r#"
            SELECT DISTINCT from_concept_id
            FROM kb_okf_concept_link
            WHERE tenant_id = $1 AND space_id = $2 AND to_concept_id = $3 AND status = $4
            ORDER BY from_concept_id ASC
            LIMIT 200
            "#,
        )
        .bind(tenant_id)
        .bind(space_id)
        .bind(to_concept_id)
        .bind(ACTIVE_STATUS)
        .fetch_all(&self.pool)
        .await
        .map_err(|error| KnowledgeOkfConceptLinkStoreError::Internal(error.to_string()))?;
        Ok(rows)
    }

    async fn list_orphan_concept_ids(
        &self,
        space_id: u64,
        published_concept_ids: &[String],
    ) -> Result<Vec<String>, KnowledgeOkfConceptLinkStoreError> {
        let tenant_id = to_i64("tenant_id", self.tenant_id)?;
        let space_id = to_i64("space_id", space_id)?;
        let inbound_targets = sqlx::query_scalar::<_, String>(
            r#"
            SELECT DISTINCT to_concept_id
            FROM kb_okf_concept_link
            WHERE tenant_id = $1 AND space_id = $2 AND status = $3
            LIMIT $4
            "#,
        )
        .bind(tenant_id)
        .bind(space_id)
        .bind(ACTIVE_STATUS)
        .bind(MAX_OKF_ORPHAN_LINK_TARGETS)
        .fetch_all(&self.pool)
        .await
        .map_err(|error| KnowledgeOkfConceptLinkStoreError::Internal(error.to_string()))?;
        let inbound: BTreeSet<String> = inbound_targets.into_iter().collect();
        Ok(published_concept_ids
            .iter()
            .filter(|concept_id| !inbound.contains(*concept_id))
            .cloned()
            .collect())
    }

    async fn list_active_link_edges(
        &self,
        space_id: u64,
    ) -> Result<Vec<KnowledgeOkfConceptLinkEdge>, KnowledgeOkfConceptLinkStoreError> {
        let tenant_id = to_i64("tenant_id", self.tenant_id)?;
        let space_id = to_i64("space_id", space_id)?;
        let rows = sqlx::query(
            r#"
            SELECT from_concept_id, to_concept_id, anchor_text
            FROM kb_okf_concept_link
            WHERE tenant_id = $1 AND space_id = $2 AND status = $3
            ORDER BY from_concept_id ASC, to_concept_id ASC, anchor_text ASC
            LIMIT 2000
            "#,
        )
        .bind(tenant_id)
        .bind(space_id)
        .bind(ACTIVE_STATUS)
        .fetch_all(&self.pool)
        .await
        .map_err(|error| KnowledgeOkfConceptLinkStoreError::Internal(error.to_string()))?;

        Ok(rows
            .into_iter()
            .map(|row| KnowledgeOkfConceptLinkEdge {
                from_concept_id: row.get("from_concept_id"),
                to_concept_id: row.get("to_concept_id"),
                anchor_text: row.get("anchor_text"),
            })
            .collect())
    }
}

fn now_rfc3339() -> Result<String, KnowledgeOkfConceptLinkStoreError> {
    OffsetDateTime::now_utc()
        .format(&Rfc3339)
        .map_err(|error| KnowledgeOkfConceptLinkStoreError::Internal(error.to_string()))
}

fn to_i64(field: &str, value: u64) -> Result<i64, KnowledgeOkfConceptLinkStoreError> {
    i64::try_from(value).map_err(|_| {
        KnowledgeOkfConceptLinkStoreError::Internal(format!("{field} is out of range"))
    })
}

fn id_error(error: crate::KnowledgeIdGeneratorError) -> KnowledgeOkfConceptLinkStoreError {
    KnowledgeOkfConceptLinkStoreError::Internal(error.to_string())
}
