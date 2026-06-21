use crate::ports::{
    knowledge_chunk_store::{CreateKnowledgeChunkRecord, KnowledgeChunkStore},
    knowledge_document_store::{
        CreateKnowledgeDocumentRecord, KnowledgeDocumentIdentityScope, KnowledgeDocumentStore,
    },
    knowledge_document_version_store::{
        CreateKnowledgeDocumentVersionRecord, KnowledgeDocumentVersionStore,
    },
    knowledge_drive_object_ref_store::{
        CreateKnowledgeDriveObjectRefRecord, KnowledgeDriveObjectRefStore,
        MANAGED_DRIVE_ACCESS_MODE, SDKWORK_DRIVE_PROVIDER_KIND,
    },
    knowledge_drive_storage::KnowledgeObjectRef,
};
use sdkwork_utils_rust::{is_blank, sha256_hash};
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MarkdownIndexResult {
    pub document_version_id: u64,
    pub chunk_count: usize,
}

pub struct KnowledgeApiMarkdownIndexService<'a> {
    documents: &'a dyn KnowledgeDocumentStore,
    versions: &'a dyn KnowledgeDocumentVersionStore,
    object_refs: &'a dyn KnowledgeDriveObjectRefStore,
    chunks: &'a dyn KnowledgeChunkStore,
}

impl<'a> KnowledgeApiMarkdownIndexService<'a> {
    pub fn new(
        documents: &'a dyn KnowledgeDocumentStore,
        versions: &'a dyn KnowledgeDocumentVersionStore,
        object_refs: &'a dyn KnowledgeDriveObjectRefStore,
        chunks: &'a dyn KnowledgeChunkStore,
    ) -> Self {
        Self {
            documents,
            versions,
            object_refs,
            chunks,
        }
    }

    pub async fn index_payload_markdown(
        &self,
        space_id: u64,
        source_id: u64,
        title: &str,
        payload_markdown: &str,
        payload_object_ref: &KnowledgeObjectRef,
    ) -> Result<MarkdownIndexResult, KnowledgeApiMarkdownIndexServiceError> {
        if space_id == 0 {
            return Err(KnowledgeApiMarkdownIndexServiceError::InvalidRequest(
                "space_id is required".to_string(),
            ));
        }
        if source_id == 0 {
            return Err(KnowledgeApiMarkdownIndexServiceError::InvalidRequest(
                "source_id is required".to_string(),
            ));
        }

        let object_ref = self
            .object_refs
            .create_or_get_object_ref(CreateKnowledgeDriveObjectRefRecord {
                space_id,
                drive_space_id: None,
                drive_node_id: None,
                logical_path: Some(payload_object_ref.logical_path.clone()),
                drive_provider_kind: SDKWORK_DRIVE_PROVIDER_KIND.to_string(),
                drive_storage_provider_id: payload_object_ref.storage_provider_id.clone(),
                drive_bucket: payload_object_ref.bucket.clone(),
                drive_object_key: payload_object_ref.object_key.clone(),
                drive_object_version: payload_object_ref.version_id.clone(),
                drive_etag: payload_object_ref.etag.clone(),
                content_type: Some(payload_object_ref.content_type.clone()),
                size_bytes: payload_object_ref.size_bytes,
                checksum_sha256_hex: payload_object_ref.checksum_sha256_hex.clone(),
                object_role: payload_object_ref.object_role.clone(),
                access_mode: MANAGED_DRIVE_ACCESS_MODE.to_string(),
            })
            .await
            .map_err(KnowledgeApiMarkdownIndexServiceError::ObjectRef)?;

        let document = self
            .documents
            .create_or_get_document(CreateKnowledgeDocumentRecord {
                space_id,
                collection_id: 0,
                source_id: Some(source_id),
                identity_scope: KnowledgeDocumentIdentityScope::SourceOnly,
                original_file_drive_node_id: None,
                title: title.to_string(),
                mime_type: Some("text/markdown".to_string()),
                language: None,
            })
            .await
            .map_err(KnowledgeApiMarkdownIndexServiceError::Document)?;

        let version = self
            .versions
            .create_or_get_document_version(CreateKnowledgeDocumentVersionRecord {
                document_id: document.id,
                version_no: 1,
                original_object_ref_id: object_ref.id,
                checksum_sha256_hex: payload_object_ref.checksum_sha256_hex.clone(),
                size_bytes: payload_object_ref.size_bytes,
                mime_type: Some("text/markdown".to_string()),
            })
            .await
            .map_err(KnowledgeApiMarkdownIndexServiceError::Version)?;

        let chunk_records =
            split_markdown_chunks(space_id, document.id, version.id, payload_markdown);
        let indexed = self
            .chunks
            .replace_version_chunks(version.id, chunk_records)
            .await
            .map_err(KnowledgeApiMarkdownIndexServiceError::Chunk)?;
        Ok(MarkdownIndexResult {
            document_version_id: version.id,
            chunk_count: indexed,
        })
    }

    pub async fn index_existing_document_version(
        &self,
        space_id: u64,
        document_id: u64,
        document_version_id: u64,
        payload_markdown: &str,
    ) -> Result<MarkdownIndexResult, KnowledgeApiMarkdownIndexServiceError> {
        if space_id == 0 || document_id == 0 || document_version_id == 0 {
            return Err(KnowledgeApiMarkdownIndexServiceError::InvalidRequest(
                "space_id, document_id, and document_version_id are required".to_string(),
            ));
        }
        if is_blank(Some(payload_markdown)) {
            return Err(KnowledgeApiMarkdownIndexServiceError::InvalidRequest(
                "payload content must not be empty".to_string(),
            ));
        }

        let chunk_records =
            split_markdown_chunks(space_id, document_id, document_version_id, payload_markdown);
        let indexed = self
            .chunks
            .replace_version_chunks(document_version_id, chunk_records)
            .await
            .map_err(KnowledgeApiMarkdownIndexServiceError::Chunk)?;
        Ok(MarkdownIndexResult {
            document_version_id,
            chunk_count: indexed,
        })
    }
}

pub fn split_markdown_chunks(
    space_id: u64,
    document_id: u64,
    document_version_id: u64,
    markdown: &str,
) -> Vec<CreateKnowledgeChunkRecord> {
    markdown
        .split("\n\n")
        .map(str::trim)
        .filter(|segment| !segment.is_empty())
        .enumerate()
        .map(|(index, content)| {
            let content_hash = format!("sha256:{}", sha256_hash(content.as_bytes()));
            CreateKnowledgeChunkRecord {
                space_id,
                collection_id: 0,
                document_id,
                document_version_id,
                chunk_index: (index + 1) as u32,
                content_text: content.to_string(),
                content_hash,
                token_count: Some(estimate_token_count(content)),
                locator: Some(format!("paragraph:{}", index + 1)),
            }
        })
        .collect()
}

fn estimate_token_count(content: &str) -> u32 {
    ((content.len() as u32) / 4).max(1)
}

#[derive(Debug, Error)]
pub enum KnowledgeApiMarkdownIndexServiceError {
    #[error("invalid markdown index request: {0}")]
    InvalidRequest(String),
    #[error(transparent)]
    ObjectRef(
        #[from] crate::ports::knowledge_drive_object_ref_store::KnowledgeDriveObjectRefStoreError,
    ),
    #[error(transparent)]
    Document(#[from] crate::ports::knowledge_document_store::KnowledgeDocumentStoreError),
    #[error(transparent)]
    Version(
        #[from] crate::ports::knowledge_document_version_store::KnowledgeDocumentVersionStoreError,
    ),
    #[error(transparent)]
    Chunk(#[from] crate::ports::knowledge_chunk_store::KnowledgeChunkStoreError),
}
