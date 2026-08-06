use async_trait::async_trait;
use sdkwork_database_config::DatabaseEngine;
use sdkwork_intelligence_knowledgebase_service::ports::drive_import_metadata_store::DriveImportMetadataStoreError;
use sdkwork_intelligence_knowledgebase_service::ports::markdown_index_metadata_store::{
    MarkdownIndexMetadataStore, MarkdownIndexMetadataStoreError, MarkdownIndexSourceBinding,
    PrepareMarkdownIndexMetadataRecord, PreparedMarkdownIndexMetadata,
};
use sdkwork_knowledgebase_observability::KnowledgebaseTenantQuotaLimits;
use sqlx::AnyPool;
use std::sync::Arc;

use crate::db::sql_timestamp::SqlTimestampDialect;
use crate::id::{default_knowledge_id_generator, KnowledgeIdGenerator};
use crate::quota_transaction::{
    begin_tenant_quota_transaction, enforce_tenant_quotas_after_write, TenantQuotaTransactionError,
};
use crate::postgres_knowledge_document_metadata_transaction::{
    bind_document_current_version_in_transaction, create_or_get_document_in_transaction,
    create_or_get_document_version_in_transaction, create_or_get_object_ref_in_transaction,
    create_or_get_source_in_transaction,
};

#[derive(Debug, Clone)]
pub struct PostgresMarkdownIndexMetadataStore {
    pool: AnyPool,
    tenant_id: u64,
    organization_id: u64,
    id_generator: Arc<dyn KnowledgeIdGenerator>,
    timestamp_dialect: SqlTimestampDialect,
    database_engine: DatabaseEngine,
    quota_limits: Option<KnowledgebaseTenantQuotaLimits>,
}

impl PostgresMarkdownIndexMetadataStore {
    pub fn new(pool: AnyPool, tenant_id: u64, organization_id: u64) -> Self {
        Self::with_id_generator(
            pool,
            tenant_id,
            organization_id,
            default_knowledge_id_generator(),
        )
    }

    pub fn with_id_generator(
        pool: AnyPool,
        tenant_id: u64,
        organization_id: u64,
        id_generator: Arc<dyn KnowledgeIdGenerator>,
    ) -> Self {
        Self {
            pool,
            tenant_id,
            organization_id,
            id_generator,
            timestamp_dialect: SqlTimestampDialect::default(),
            database_engine: DatabaseEngine::Postgres,
            quota_limits: None,
        }
    }

    pub fn with_database_engine(mut self, database_engine: DatabaseEngine) -> Self {
        self.timestamp_dialect = SqlTimestampDialect::from_database_engine(database_engine);
        self.database_engine = database_engine;
        self
    }

    pub fn with_quota_limits(mut self, quota_limits: KnowledgebaseTenantQuotaLimits) -> Self {
        self.quota_limits = Some(quota_limits);
        self
    }
}

#[async_trait]
impl MarkdownIndexMetadataStore for PostgresMarkdownIndexMetadataStore {
    async fn create_or_prepare_markdown_index_metadata(
        &self,
        record: PrepareMarkdownIndexMetadataRecord,
    ) -> Result<PreparedMarkdownIndexMetadata, MarkdownIndexMetadataStoreError> {
        postgres_create_or_prepare_markdown_index_metadata(self, record).await
    }
}

async fn postgres_create_or_prepare_markdown_index_metadata(
    store: &PostgresMarkdownIndexMetadataStore,
    record: PrepareMarkdownIndexMetadataRecord,
) -> Result<PreparedMarkdownIndexMetadata, MarkdownIndexMetadataStoreError> {
    use sdkwork_intelligence_knowledgebase_service::ports::markdown_index_metadata_store::validate_markdown_index_document_identity;

    let pool = &store.pool;
    let tenant_id = store.tenant_id;
    let organization_id = store.organization_id;
    let id_generator = &store.id_generator;
    let timestamp_dialect = store.timestamp_dialect;
    let quota_limits = store.quota_limits;

    validate_markdown_index_document_identity(&record.document, &record.source)?;

    let tenant_id_i64 = i64::try_from(tenant_id).map_err(|_| {
        MarkdownIndexMetadataStoreError::invalid_request("tenant_id exceeds i64 range")
    })?;
    let organization_id_i64 = i64::try_from(organization_id).map_err(|_| {
        MarkdownIndexMetadataStoreError::invalid_request("organization_id exceeds i64 range")
    })?;
    let mut transaction = if quota_limits.is_some() {
        begin_tenant_quota_transaction(pool, tenant_id_i64, organization_id_i64)
            .await
            .map_err(|error| MarkdownIndexMetadataStoreError::internal(error.to_string()))?
    } else {
        pool.begin()
            .await
            .map_err(|error| MarkdownIndexMetadataStoreError::internal(error.to_string()))?
    };

    let source_id = match &record.source {
        MarkdownIndexSourceBinding::Existing { source_id } => *source_id,
        MarkdownIndexSourceBinding::Create(source_record) => {
            create_or_get_source_in_transaction(
                &mut transaction,
                tenant_id,
                organization_id,
                id_generator,
                timestamp_dialect,
                source_record,
            )
            .await
            .map_err(map_markdown_index_metadata_error)?
            .id
        }
    };

    let object_ref = create_or_get_object_ref_in_transaction(
        &mut transaction,
        tenant_id,
        organization_id,
        id_generator,
        timestamp_dialect,
        &record.object_ref,
    )
    .await
    .map_err(map_markdown_index_metadata_error)?;

    let mut document_record = record.document;
    document_record.source_id = Some(source_id);
    let document = create_or_get_document_in_transaction(
        &mut transaction,
        tenant_id,
        organization_id,
        id_generator,
        timestamp_dialect,
        &document_record,
    )
    .await
    .map_err(map_markdown_index_metadata_error)?;

    let mut version_record = record.version;
    version_record.document_id = document.id;
    version_record.original_object_ref_id = object_ref.id;
    let version = create_or_get_document_version_in_transaction(
        &mut transaction,
        tenant_id,
        organization_id,
        id_generator,
        timestamp_dialect,
        &version_record,
    )
    .await
    .map_err(map_markdown_index_metadata_error)?;

    bind_document_current_version_in_transaction(
        &mut transaction,
        tenant_id,
        organization_id,
        timestamp_dialect,
        document.id,
        version.id,
        version.version_no,
    )
    .await
    .map_err(map_markdown_index_metadata_error)?;

    if let Some(limits) = quota_limits {
        enforce_tenant_quotas_after_write(
            &mut transaction,
                        tenant_id_i64,
            organization_id_i64,
            limits,
        )
        .await
        .map_err(map_quota_transaction_error)?;
    }

    transaction
        .commit()
        .await
        .map_err(|error| MarkdownIndexMetadataStoreError::internal(error.to_string()))?;

    Ok(PreparedMarkdownIndexMetadata {
        source_id,
        object_ref,
        document,
        version,
    })
}

fn map_markdown_index_metadata_error(
    error: DriveImportMetadataStoreError,
) -> MarkdownIndexMetadataStoreError {
    match error {
        DriveImportMetadataStoreError::InvalidRequest(detail) => {
            MarkdownIndexMetadataStoreError::invalid_request(detail)
        }
        DriveImportMetadataStoreError::Conflict(detail) => {
            MarkdownIndexMetadataStoreError::Conflict(detail)
        }
        DriveImportMetadataStoreError::QuotaExceeded(error) => {
            MarkdownIndexMetadataStoreError::QuotaExceeded(error)
        }
        DriveImportMetadataStoreError::Internal(detail) => {
            MarkdownIndexMetadataStoreError::internal(detail)
        }
    }
}

fn map_quota_transaction_error(
    error: TenantQuotaTransactionError,
) -> MarkdownIndexMetadataStoreError {
    match error {
        TenantQuotaTransactionError::Quota(error) => {
            MarkdownIndexMetadataStoreError::QuotaExceeded(error)
        }
        TenantQuotaTransactionError::Database(error) => {
            MarkdownIndexMetadataStoreError::internal(error.to_string())
        }
        TenantQuotaTransactionError::Invalid(detail) => {
            MarkdownIndexMetadataStoreError::internal(detail)
        }
    }
}
