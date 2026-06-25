use async_trait::async_trait;
use sdkwork_intelligence_knowledgebase_service::ports::drive_import_metadata_store::DriveImportMetadataStoreError;
use sdkwork_intelligence_knowledgebase_service::ports::markdown_index_metadata_store::{
    MarkdownIndexMetadataStore, MarkdownIndexMetadataStoreError, MarkdownIndexSourceBinding,
    PrepareMarkdownIndexMetadataRecord, PreparedMarkdownIndexMetadata,
};
use sqlx::AnyPool;
use std::sync::Arc;

use crate::id::{default_knowledge_id_generator, KnowledgeIdGenerator};
use crate::sqlite_knowledge_document_metadata_transaction::{
    bind_document_current_version_in_transaction, create_or_get_document_in_transaction,
    create_or_get_document_version_in_transaction, create_or_get_object_ref_in_transaction,
    create_or_get_source_in_transaction,
};

#[derive(Debug, Clone)]
pub struct SqliteMarkdownIndexMetadataStore {
    pool: AnyPool,
    tenant_id: u64,
    id_generator: Arc<dyn KnowledgeIdGenerator>,
}

impl SqliteMarkdownIndexMetadataStore {
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
impl MarkdownIndexMetadataStore for SqliteMarkdownIndexMetadataStore {
    async fn create_or_prepare_markdown_index_metadata(
        &self,
        record: PrepareMarkdownIndexMetadataRecord,
    ) -> Result<PreparedMarkdownIndexMetadata, MarkdownIndexMetadataStoreError> {
        sqlite_create_or_prepare_markdown_index_metadata(
            &self.pool,
            self.tenant_id,
            &self.id_generator,
            record,
        )
        .await
    }
}

async fn sqlite_create_or_prepare_markdown_index_metadata(
    pool: &AnyPool,
    tenant_id: u64,
    id_generator: &Arc<dyn KnowledgeIdGenerator>,
    record: PrepareMarkdownIndexMetadataRecord,
) -> Result<PreparedMarkdownIndexMetadata, MarkdownIndexMetadataStoreError> {
    use sdkwork_intelligence_knowledgebase_service::ports::markdown_index_metadata_store::validate_markdown_index_document_identity;

    validate_markdown_index_document_identity(&record.document, &record.source)?;

    let mut transaction = pool
        .begin()
        .await
        .map_err(|error| MarkdownIndexMetadataStoreError::internal(error.to_string()))?;

    let source_id = match &record.source {
        MarkdownIndexSourceBinding::Existing { source_id } => *source_id,
        MarkdownIndexSourceBinding::Create(source_record) => {
            create_or_get_source_in_transaction(
                &mut transaction,
                tenant_id,
                id_generator,
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
        id_generator,
        &record.object_ref,
    )
    .await
    .map_err(map_markdown_index_metadata_error)?;

    let mut document_record = record.document;
    document_record.source_id = Some(source_id);
    let document = create_or_get_document_in_transaction(
        &mut transaction,
        tenant_id,
        id_generator,
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
        id_generator,
        &version_record,
    )
    .await
    .map_err(map_markdown_index_metadata_error)?;

    bind_document_current_version_in_transaction(
        &mut transaction,
        tenant_id,
        document.id,
        version.id,
        version.version_no,
    )
    .await
    .map_err(map_markdown_index_metadata_error)?;

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
        DriveImportMetadataStoreError::Internal(detail) => {
            MarkdownIndexMetadataStoreError::internal(detail)
        }
    }
}
