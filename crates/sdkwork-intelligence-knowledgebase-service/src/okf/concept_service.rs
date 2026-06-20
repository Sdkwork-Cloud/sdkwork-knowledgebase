use crate::okf::{
    render_index_md, render_log_md, validate_concept_document, validate_concept_id,
    OkfConceptDocument, OkfConformanceError, render_okf_concept_markdown,
};
use crate::ports::{
    knowledge_drive_object_ref_store::{
        CreateKnowledgeDriveObjectRefRecord, KnowledgeDriveObjectRefStore,
        KnowledgeDriveObjectRefStoreError, MANAGED_DRIVE_ACCESS_MODE, SDKWORK_DRIVE_PROVIDER_KIND,
    },
    knowledge_drive_storage::{
        KnowledgeDriveStorage, KnowledgeObjectRef, KnowledgeStorageError, PutKnowledgeObjectRequest,
    },
    knowledge_drive_workspace::{
        EnsureKnowledgeDriveNodeKind, EnsureKnowledgeDriveNodeRequest,
        EnsureKnowledgeDriveNodesRequest, KnowledgeDriveWorkspace, KnowledgeDriveWorkspaceError,
    },
    knowledge_okf_bundle_file_store::{
        CreateKnowledgeOkfBundleFileRecord, KnowledgeOkfBundleFileStore,
        KnowledgeOkfBundleFileStoreError,
    },
    knowledge_okf_concept_store::{
        AppendKnowledgeOkfLogEntryRecord, CreateKnowledgeOkfConceptRevisionRecord,
        KnowledgeOkfConceptStore, KnowledgeOkfConceptStoreError,
        MarkKnowledgeOkfConceptCurrentRevisionRecord, UpsertKnowledgeOkfConceptRecord,
    },
};
use sdkwork_knowledgebase_contract::{
    okf::{
        KnowledgeOkfConceptPublication, OkfBundlePaths, OkfConceptPublishState, OkfLogEventType,
        OkfRevisionReviewState, PublishKnowledgeOkfConceptRequest,
    },
    okf_bundle_file::OkfBundleFileKind,
};
use sha2::{Digest, Sha256};
use thiserror::Error;
use time::format_description::well_known::Rfc3339;
use time::OffsetDateTime;

pub struct OkfConceptService<'a> {
    drive: &'a dyn KnowledgeDriveStorage,
    object_refs: &'a dyn KnowledgeDriveObjectRefStore,
    concept_store: &'a dyn KnowledgeOkfConceptStore,
    file_entries: Option<&'a dyn KnowledgeOkfBundleFileStore>,
    drive_workspace: Option<&'a dyn KnowledgeDriveWorkspace>,
}

impl<'a> OkfConceptService<'a> {
    pub fn new(
        drive: &'a dyn KnowledgeDriveStorage,
        object_refs: &'a dyn KnowledgeDriveObjectRefStore,
        concept_store: &'a dyn KnowledgeOkfConceptStore,
    ) -> Self {
        Self {
            drive,
            object_refs,
            concept_store,
            file_entries: None,
            drive_workspace: None,
        }
    }

    pub fn with_file_entry_store(
        mut self,
        file_entries: &'a dyn KnowledgeOkfBundleFileStore,
    ) -> Self {
        self.file_entries = Some(file_entries);
        self
    }

    pub fn with_drive_workspace(
        mut self,
        drive_workspace: &'a dyn KnowledgeDriveWorkspace,
    ) -> Self {
        self.drive_workspace = Some(drive_workspace);
        self
    }

    pub async fn publish_concept(
        &self,
        request: PublishKnowledgeOkfConceptRequest,
        drive_space_id: Option<&str>,
    ) -> Result<KnowledgeOkfConceptPublication, OkfConceptServiceError> {
        validate_publish_request(&request)?;
        let drive_space_id = self.required_drive_space_id(drive_space_id)?;
        let concept_id = normalize_concept_id(&request.concept_id)?;
        let published_logical_path = OkfBundlePaths::concept_logical_path(&concept_id);
        let concept_document = OkfConceptDocument {
            concept_type: request.concept_type.clone(),
            title: Some(request.title.clone()),
            description: Some(request.description.clone()),
            resource: None,
            tags: request.tags.clone(),
            timestamp: Some(now_rfc3339()?),
            body: request.markdown.clone(),
        };
        validate_concept_document(&concept_document, &concept_id)
            .map_err(OkfConceptServiceError::Conformance)?;
        let published_markdown = render_okf_concept_markdown(&concept_document);

        let concept = self
            .concept_store
            .upsert_concept(UpsertKnowledgeOkfConceptRecord {
                space_id: request.space_id,
                concept_id: concept_id.clone(),
                title: request.title.clone(),
                concept_type: request.concept_type.clone(),
                logical_path: published_logical_path.clone(),
                description: request.description.clone(),
                source_count: request.source_count,
                tags: request.tags.clone(),
                publish_state: OkfConceptPublishState::CandidateReady,
            })
            .await?;
        let revision_no = self.concept_store.next_revision_no(concept.id).await?;
        let governance_revision_path =
            format!(".sdkwork/governance/revisions/{concept_id}/r{revision_no}.md");

        let published_ref = self
            .put_markdown(
                &published_logical_path,
                "concept_revision",
                &published_markdown,
            )
            .await?;
        let revision_ref = self
            .put_markdown(
                &governance_revision_path,
                "concept_revision",
                &published_markdown,
            )
            .await?;
        self.object_refs
            .create_or_get_object_ref(object_ref_record(
                request.space_id,
                drive_space_id.as_deref(),
                None,
                &published_ref,
            ))
            .await?;
        let markdown_object_ref = self
            .object_refs
            .create_or_get_object_ref(object_ref_record(
                request.space_id,
                drive_space_id.as_deref(),
                None,
                &revision_ref,
            ))
            .await?;
        let revision = self
            .concept_store
            .create_revision(CreateKnowledgeOkfConceptRevisionRecord {
                concept_row_id: concept.id,
                revision_no,
                markdown_object_ref_id: markdown_object_ref.id,
                content_hash: checksum_sha256_hex(published_markdown.as_bytes()),
                review_state: OkfRevisionReviewState::Approved,
            })
            .await?;
        let concept = self
            .concept_store
            .mark_current_revision(MarkKnowledgeOkfConceptCurrentRevisionRecord {
                concept_row_id: concept.id,
                revision_id: revision.id,
                publish_state: OkfConceptPublishState::Published,
            })
            .await?;

        self.ensure_drive_nodes(
            drive_space_id.as_deref(),
            [
                folder_node(".sdkwork/governance/revisions"),
                folder_node(&format!(".sdkwork/governance/revisions/{concept_id}")),
                file_node(&published_ref),
                file_node(&revision_ref),
            ],
        )
        .await?;

        self.concept_store
            .append_log_entry(AppendKnowledgeOkfLogEntryRecord {
                space_id: request.space_id,
                event_type: OkfLogEventType::Publish.as_str().to_string(),
                event_time: now_rfc3339()?,
                title: format!("Published {}", request.title),
                actor: request.actor,
                affected_concepts: vec![request.title],
                audit_event_id: None,
                warnings: vec![],
                privacy_level: "internal".to_string(),
            })
            .await?;
        self.rebuild_standard_files(request.space_id, drive_space_id.as_deref())
            .await?;

        Ok(KnowledgeOkfConceptPublication {
            concept,
            revision,
            published_logical_path,
            governance_revision_path,
        })
    }

    async fn put_markdown(
        &self,
        logical_path: &str,
        object_role: &str,
        markdown: &str,
    ) -> Result<KnowledgeObjectRef, OkfConceptServiceError> {
        Ok(self
            .drive
            .put_object(PutKnowledgeObjectRequest::text(
                logical_path.to_string(),
                object_role.to_string(),
                markdown.to_string(),
                None,
            ))
            .await?)
    }

    async fn rebuild_standard_files(
        &self,
        space_id: u64,
        drive_space_id: Option<&str>,
    ) -> Result<(), OkfConceptServiceError> {
        let Some(file_entries) = self.file_entries else {
            return Ok(());
        };
        let paths = OkfBundlePaths::default();
        let summaries = self.concept_store.list_concept_summaries(space_id).await?;
        let logs = self.concept_store.list_log_entries(space_id).await?;
        let index_ref = self
            .put_markdown(
                paths.index_md,
                "bundle_index",
                &render_index_md("Knowledge Space", &summaries),
            )
            .await?;
        let log_ref = self
            .put_markdown(paths.log_md, "bundle_log", &render_log_md(&logs))
            .await?;
        upsert_file_entry(
            file_entries,
            space_id,
            &index_ref,
            OkfBundleFileKind::BundleIndex,
        )
        .await?;
        upsert_file_entry(
            file_entries,
            space_id,
            &log_ref,
            OkfBundleFileKind::BundleLog,
        )
        .await?;
        self.ensure_drive_nodes(drive_space_id, [file_node(&index_ref), file_node(&log_ref)])
            .await?;
        Ok(())
    }

    async fn ensure_drive_nodes<const N: usize>(
        &self,
        drive_space_id: Option<&str>,
        nodes: [EnsureKnowledgeDriveNodeRequest; N],
    ) -> Result<(), OkfConceptServiceError> {
        let Some(workspace) = self.drive_workspace else {
            return Ok(());
        };
        let drive_space_id = drive_space_id
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .ok_or_else(|| {
                OkfConceptServiceError::InvalidRequest(
                    "drive_space_id is required when drive workspace synchronization is enabled"
                        .to_string(),
                )
            })?;
        workspace
            .ensure_nodes(EnsureKnowledgeDriveNodesRequest {
                drive_space_id: drive_space_id.to_string(),
                nodes: nodes.into_iter().collect(),
            })
            .await?;
        Ok(())
    }

    fn required_drive_space_id(
        &self,
        drive_space_id: Option<&str>,
    ) -> Result<Option<String>, OkfConceptServiceError> {
        if self.drive_workspace.is_none() {
            return Ok(None);
        }
        drive_space_id
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(ToOwned::to_owned)
            .map(Some)
            .ok_or_else(|| {
                OkfConceptServiceError::InvalidRequest(
                    "drive_space_id is required when drive workspace synchronization is enabled"
                        .to_string(),
                )
            })
    }
}

async fn upsert_file_entry(
    file_entries: &dyn KnowledgeOkfBundleFileStore,
    space_id: u64,
    object_ref: &KnowledgeObjectRef,
    file_kind: OkfBundleFileKind,
) -> Result<(), OkfConceptServiceError> {
    file_entries
        .upsert_file_entry(CreateKnowledgeOkfBundleFileRecord {
            space_id,
            logical_path: object_ref.logical_path.clone(),
            file_kind,
            artifact_role: object_ref.object_role.clone(),
            drive_bucket: object_ref.bucket.clone(),
            drive_object_key: object_ref.object_key.clone(),
            checksum_sha256_hex: object_ref.checksum_sha256_hex.clone(),
        })
        .await?;
    Ok(())
}

fn object_ref_record(
    space_id: u64,
    drive_space_id: Option<&str>,
    drive_node_id: Option<String>,
    object_ref: &KnowledgeObjectRef,
) -> CreateKnowledgeDriveObjectRefRecord {
    CreateKnowledgeDriveObjectRefRecord {
        space_id,
        drive_space_id: drive_space_id.map(str::to_string),
        drive_node_id,
        logical_path: Some(object_ref.logical_path.clone()),
        drive_provider_kind: SDKWORK_DRIVE_PROVIDER_KIND.to_string(),
        drive_storage_provider_id: object_ref.storage_provider_id.clone(),
        drive_bucket: object_ref.bucket.clone(),
        drive_object_key: object_ref.object_key.clone(),
        drive_object_version: object_ref.version_id.clone(),
        drive_etag: object_ref.etag.clone(),
        content_type: Some(object_ref.content_type.clone()),
        size_bytes: object_ref.size_bytes,
        checksum_sha256_hex: object_ref.checksum_sha256_hex.clone(),
        object_role: object_ref.object_role.clone(),
        access_mode: MANAGED_DRIVE_ACCESS_MODE.to_string(),
    }
}

fn file_node(object_ref: &KnowledgeObjectRef) -> EnsureKnowledgeDriveNodeRequest {
    EnsureKnowledgeDriveNodeRequest {
        logical_path: object_ref.logical_path.clone(),
        kind: EnsureKnowledgeDriveNodeKind::File,
        object_ref: Some(object_ref.clone()),
    }
}

fn folder_node(logical_path: &str) -> EnsureKnowledgeDriveNodeRequest {
    EnsureKnowledgeDriveNodeRequest {
        logical_path: logical_path.to_string(),
        kind: EnsureKnowledgeDriveNodeKind::Folder,
        object_ref: None,
    }
}

fn validate_publish_request(
    request: &PublishKnowledgeOkfConceptRequest,
) -> Result<(), OkfConceptServiceError> {
    if request.space_id == 0 {
        return Err(OkfConceptServiceError::InvalidRequest(
            "space_id is required".to_string(),
        ));
    }
    if request.title.trim().is_empty() {
        return Err(OkfConceptServiceError::InvalidRequest(
            "title is required".to_string(),
        ));
    }
    if request.markdown.trim().is_empty() {
        return Err(OkfConceptServiceError::InvalidRequest(
            "markdown is required".to_string(),
        ));
    }
    safe_concept_id(&request.concept_id)?;
    Ok(())
}

fn normalize_concept_id(value: &str) -> Result<String, OkfConceptServiceError> {
    validate_concept_id(value).map_err(OkfConceptServiceError::Conformance)?;
    Ok(value.trim().to_string())
}

fn safe_concept_id(value: &str) -> Result<String, OkfConceptServiceError> {
    normalize_concept_id(value)
}

fn now_rfc3339() -> Result<String, OkfConceptServiceError> {
    OffsetDateTime::now_utc()
        .format(&Rfc3339)
        .map_err(|error| OkfConceptServiceError::Internal(error.to_string()))
}

fn checksum_sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    let mut output = String::with_capacity(digest.len() * 2);
    for byte in digest {
        output.push_str(&format!("{byte:02x}"));
    }
    output
}

#[derive(Debug, Error)]
pub enum OkfConceptServiceError {
    #[error("invalid knowledge okf concept request: {0}")]
    InvalidRequest(String),
    #[error(transparent)]
    Conformance(#[from] OkfConformanceError),
    #[error(transparent)]
    Storage(#[from] KnowledgeStorageError),
    #[error(transparent)]
    ObjectRefStore(#[from] KnowledgeDriveObjectRefStoreError),
    #[error(transparent)]
    ConceptStore(#[from] KnowledgeOkfConceptStoreError),
    #[error(transparent)]
    FileEntryStore(#[from] KnowledgeOkfBundleFileStoreError),
    #[error(transparent)]
    DriveWorkspace(#[from] KnowledgeDriveWorkspaceError),
    #[error("knowledge okf concept internal error: {0}")]
    Internal(String),
}
