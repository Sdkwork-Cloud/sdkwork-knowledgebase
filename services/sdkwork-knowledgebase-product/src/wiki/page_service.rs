use crate::ports::knowledge_drive_object_ref_store::{
    CreateKnowledgeDriveObjectRefRecord, KnowledgeDriveObjectRefStore,
    KnowledgeDriveObjectRefStoreError, MANAGED_DRIVE_ACCESS_MODE, SDKWORK_DRIVE_PROVIDER_KIND,
};
use crate::ports::knowledge_drive_storage::{
    KnowledgeDriveStorage, KnowledgeObjectRef, KnowledgeStorageError, PutKnowledgeObjectRequest,
};
use crate::ports::knowledge_drive_workspace::{
    EnsureKnowledgeDriveNodeKind, EnsureKnowledgeDriveNodeRequest,
    EnsureKnowledgeDriveNodesRequest, KnowledgeDriveWorkspace, KnowledgeDriveWorkspaceError,
};
use crate::ports::knowledge_wiki_file_entry_store::{
    CreateKnowledgeWikiFileEntryRecord, KnowledgeWikiFileEntryStore,
    KnowledgeWikiFileEntryStoreError,
};
use crate::ports::knowledge_wiki_page_store::{
    AppendKnowledgeWikiLogEntryRecord, CreateKnowledgeWikiPageRevisionRecord,
    KnowledgeWikiPageStore, KnowledgeWikiPageStoreError, MarkKnowledgeWikiCurrentRevisionRecord,
    UpsertKnowledgeWikiPageRecord,
};
use crate::wiki::{render_index_md, render_log_md};
use sdkwork_knowledgebase_contract::wiki::{
    KnowledgeWikiPagePublication, PublishKnowledgeWikiPageRequest, WikiLogEventType,
    WikiPagePublishState, WikiRevisionReviewState,
};
use sdkwork_knowledgebase_contract::wiki_file::WikiFileEntryType;
use sha2::{Digest, Sha256};
use thiserror::Error;
use time::format_description::well_known::Rfc3339;
use time::OffsetDateTime;

pub struct KnowledgeWikiPageService<'a> {
    drive: &'a dyn KnowledgeDriveStorage,
    object_refs: &'a dyn KnowledgeDriveObjectRefStore,
    pages: &'a dyn KnowledgeWikiPageStore,
    file_entries: Option<&'a dyn KnowledgeWikiFileEntryStore>,
    drive_workspace: Option<&'a dyn KnowledgeDriveWorkspace>,
}

impl<'a> KnowledgeWikiPageService<'a> {
    pub fn new(
        drive: &'a dyn KnowledgeDriveStorage,
        object_refs: &'a dyn KnowledgeDriveObjectRefStore,
        pages: &'a dyn KnowledgeWikiPageStore,
    ) -> Self {
        Self {
            drive,
            object_refs,
            pages,
            file_entries: None,
            drive_workspace: None,
        }
    }

    pub fn with_file_entry_store(
        mut self,
        file_entries: &'a dyn KnowledgeWikiFileEntryStore,
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

    pub async fn publish_page(
        &self,
        request: PublishKnowledgeWikiPageRequest,
        drive_space_id: Option<&str>,
    ) -> Result<KnowledgeWikiPagePublication, KnowledgeWikiPageServiceError> {
        validate_publish_request(&request)?;
        let drive_space_id = self.required_drive_space_id(drive_space_id)?;
        let page_dir = wiki_page_dir(request.page_type, &request.slug)?;
        let current_file_path = format!("{page_dir}/current.md");

        let page = self
            .pages
            .upsert_page(UpsertKnowledgeWikiPageRecord {
                space_id: request.space_id,
                slug: request.slug.clone(),
                title: request.title.clone(),
                page_type: request.page_type,
                logical_path: current_file_path.clone(),
                summary: request.summary.clone(),
                source_count: request.source_count,
                tags: request.tags.clone(),
                publish_state: WikiPagePublishState::CandidateReady,
            })
            .await?;
        let revision_no = self.pages.next_revision_no(page.id).await?;
        let revision_file_path = format!("{page_dir}/revisions/r{revision_no}.md");

        let current_ref = self
            .put_markdown(&current_file_path, "wiki_revision", &request.markdown)
            .await?;
        let revision_ref = self
            .put_markdown(&revision_file_path, "wiki_revision", &request.markdown)
            .await?;
        self.object_refs
            .create_or_get_object_ref(object_ref_record(
                request.space_id,
                drive_space_id.as_deref(),
                None,
                &current_ref,
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
            .pages
            .create_revision(CreateKnowledgeWikiPageRevisionRecord {
                page_id: page.id,
                revision_no,
                markdown_object_ref_id: markdown_object_ref.id,
                content_hash: checksum_sha256_hex(request.markdown.as_bytes()),
                review_state: WikiRevisionReviewState::Approved,
            })
            .await?;
        let page = self
            .pages
            .mark_current_revision(MarkKnowledgeWikiCurrentRevisionRecord {
                page_id: page.id,
                revision_id: revision.id,
                publish_state: WikiPagePublishState::Published,
            })
            .await?;

        self.ensure_drive_nodes(
            drive_space_id.as_deref(),
            [
                folder_node(&page_dir),
                folder_node(&format!("{page_dir}/revisions")),
                file_node(&current_ref),
                file_node(&revision_ref),
            ],
        )
        .await?;

        self.pages
            .append_log_entry(AppendKnowledgeWikiLogEntryRecord {
                space_id: request.space_id,
                event_type: WikiLogEventType::Publish.as_str().to_string(),
                event_time: now_rfc3339()?,
                title: format!("Published {}", request.title),
                actor: request.actor,
                affected_pages: vec![request.title],
                audit_event_id: None,
                warnings: vec![],
                privacy_level: "internal".to_string(),
            })
            .await?;
        self.rebuild_standard_files(request.space_id, drive_space_id.as_deref())
            .await?;

        Ok(KnowledgeWikiPagePublication {
            page,
            revision,
            current_file_path,
            revision_file_path,
        })
    }

    async fn put_markdown(
        &self,
        logical_path: &str,
        object_role: &str,
        markdown: &str,
    ) -> Result<KnowledgeObjectRef, KnowledgeWikiPageServiceError> {
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
    ) -> Result<(), KnowledgeWikiPageServiceError> {
        let Some(file_entries) = self.file_entries else {
            return Ok(());
        };
        let pages = self.pages.list_page_summaries(space_id).await?;
        let logs = self.pages.list_log_entries(space_id).await?;
        let index_ref = self
            .put_markdown(
                "wiki/index.md",
                "wiki_index",
                &render_index_md("Knowledge Space", &pages),
            )
            .await?;
        let log_ref = self
            .put_markdown("wiki/log.md", "wiki_log", &render_log_md(&logs))
            .await?;
        upsert_file_entry(
            file_entries,
            space_id,
            &index_ref,
            WikiFileEntryType::WikiIndex,
        )
        .await?;
        upsert_file_entry(file_entries, space_id, &log_ref, WikiFileEntryType::WikiLog).await?;
        self.ensure_drive_nodes(drive_space_id, [file_node(&index_ref), file_node(&log_ref)])
            .await?;
        Ok(())
    }

    async fn ensure_drive_nodes<const N: usize>(
        &self,
        drive_space_id: Option<&str>,
        nodes: [EnsureKnowledgeDriveNodeRequest; N],
    ) -> Result<(), KnowledgeWikiPageServiceError> {
        let Some(workspace) = self.drive_workspace else {
            return Ok(());
        };
        let drive_space_id = drive_space_id
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .ok_or_else(|| {
                KnowledgeWikiPageServiceError::InvalidRequest(
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
    ) -> Result<Option<String>, KnowledgeWikiPageServiceError> {
        if self.drive_workspace.is_none() {
            return Ok(None);
        }
        drive_space_id
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(ToOwned::to_owned)
            .map(Some)
            .ok_or_else(|| {
                KnowledgeWikiPageServiceError::InvalidRequest(
                    "drive_space_id is required when drive workspace synchronization is enabled"
                        .to_string(),
                )
            })
    }
}

async fn upsert_file_entry(
    file_entries: &dyn KnowledgeWikiFileEntryStore,
    space_id: u64,
    object_ref: &KnowledgeObjectRef,
    entry_type: WikiFileEntryType,
) -> Result<(), KnowledgeWikiPageServiceError> {
    file_entries
        .upsert_file_entry(CreateKnowledgeWikiFileEntryRecord {
            space_id,
            logical_path: object_ref.logical_path.clone(),
            entry_type,
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

fn wiki_page_dir(
    page_type: sdkwork_knowledgebase_contract::wiki::WikiPageType,
    slug: &str,
) -> Result<String, KnowledgeWikiPageServiceError> {
    let slug = safe_slug(slug)?;
    Ok(format!("wiki/pages/{}/{}", page_type_dir(page_type), slug))
}

fn page_type_dir(page_type: sdkwork_knowledgebase_contract::wiki::WikiPageType) -> &'static str {
    match page_type {
        sdkwork_knowledgebase_contract::wiki::WikiPageType::Source => "sources",
        sdkwork_knowledgebase_contract::wiki::WikiPageType::Entity => "entities",
        sdkwork_knowledgebase_contract::wiki::WikiPageType::Topic => "topics",
        sdkwork_knowledgebase_contract::wiki::WikiPageType::Concept => "concepts",
        sdkwork_knowledgebase_contract::wiki::WikiPageType::HowTo => "how_to",
        sdkwork_knowledgebase_contract::wiki::WikiPageType::Reference => "references",
        sdkwork_knowledgebase_contract::wiki::WikiPageType::Faq => "faq",
        sdkwork_knowledgebase_contract::wiki::WikiPageType::Glossary => "glossary",
        sdkwork_knowledgebase_contract::wiki::WikiPageType::Answer => "answers",
        sdkwork_knowledgebase_contract::wiki::WikiPageType::Comparison => "comparisons",
        sdkwork_knowledgebase_contract::wiki::WikiPageType::Presentation => "presentations",
        sdkwork_knowledgebase_contract::wiki::WikiPageType::Chart => "charts",
        sdkwork_knowledgebase_contract::wiki::WikiPageType::Index => "indexes",
        sdkwork_knowledgebase_contract::wiki::WikiPageType::Policy => "policies",
        sdkwork_knowledgebase_contract::wiki::WikiPageType::Runbook => "runbooks",
    }
}

fn validate_publish_request(
    request: &PublishKnowledgeWikiPageRequest,
) -> Result<(), KnowledgeWikiPageServiceError> {
    if request.space_id == 0 {
        return Err(KnowledgeWikiPageServiceError::InvalidRequest(
            "space_id is required".to_string(),
        ));
    }
    if request.title.trim().is_empty() {
        return Err(KnowledgeWikiPageServiceError::InvalidRequest(
            "title is required".to_string(),
        ));
    }
    if request.markdown.trim().is_empty() {
        return Err(KnowledgeWikiPageServiceError::InvalidRequest(
            "markdown is required".to_string(),
        ));
    }
    safe_slug(&request.slug)?;
    Ok(())
}

fn safe_slug(value: &str) -> Result<String, KnowledgeWikiPageServiceError> {
    let value = value.trim();
    if value.is_empty()
        || value.len() > 256
        || value.starts_with('.')
        || !value
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || ch == '-' || ch == '_')
    {
        return Err(KnowledgeWikiPageServiceError::InvalidRequest(
            "slug must contain only ASCII letters, digits, hyphen, or underscore".to_string(),
        ));
    }
    Ok(value.to_string())
}

fn now_rfc3339() -> Result<String, KnowledgeWikiPageServiceError> {
    OffsetDateTime::now_utc()
        .format(&Rfc3339)
        .map_err(|error| KnowledgeWikiPageServiceError::Internal(error.to_string()))
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
pub enum KnowledgeWikiPageServiceError {
    #[error("invalid knowledge wiki page request: {0}")]
    InvalidRequest(String),
    #[error(transparent)]
    Storage(#[from] KnowledgeStorageError),
    #[error(transparent)]
    ObjectRefStore(#[from] KnowledgeDriveObjectRefStoreError),
    #[error(transparent)]
    PageStore(#[from] KnowledgeWikiPageStoreError),
    #[error(transparent)]
    FileEntryStore(#[from] KnowledgeWikiFileEntryStoreError),
    #[error(transparent)]
    DriveWorkspace(#[from] KnowledgeDriveWorkspaceError),
    #[error("knowledge wiki page internal error: {0}")]
    Internal(String),
}
