use crate::okf::{
    index_concept_links, parse_okf_markdown, render_index_md, render_log_md,
    render_okf_concept_markdown, storage::read_managed_markdown, validate_concept_document,
    validate_concept_id, OkfConceptDocument, OkfConformanceError,
};
use crate::ports::knowledge_okf_candidate_store::{
    KnowledgeOkfCandidateStore, KnowledgeOkfCandidateStoreError, UpsertKnowledgeOkfCandidateRecord,
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
    knowledge_okf_concept_link_store::{
        KnowledgeOkfConceptLinkRecord, KnowledgeOkfConceptLinkStore,
        KnowledgeOkfConceptLinkStoreError, ReplaceKnowledgeOkfConceptLinksRecord,
    },
    knowledge_okf_concept_store::{
        AppendKnowledgeOkfLogEntryRecord, CreateKnowledgeOkfConceptRevisionRecord,
        KnowledgeOkfConceptStore, KnowledgeOkfConceptStoreError,
        MarkKnowledgeOkfConceptCurrentRevisionRecord, UpsertKnowledgeOkfConceptRecord,
    },
};
use sdkwork_knowledgebase_contract::{
    okf::{
        KnowledgeOkfConcept, KnowledgeOkfConceptPublication, KnowledgeOkfConceptRevision,
        OkfBundlePaths, OkfConceptPublishState, OkfConceptUpsertRequest, OkfLogEventType,
        OkfRevisionReviewState, PublishKnowledgeOkfConceptRequest,
    },
    okf_bundle_file::OkfBundleFileKind,
    OkfCandidateType,
};
use sha2::{Digest, Sha256};
use thiserror::Error;
use time::format_description::well_known::Rfc3339;
use time::OffsetDateTime;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PublishExistingOkfConceptRevisionRequest {
    pub space_id: u64,
    pub concept: KnowledgeOkfConcept,
    pub revision: KnowledgeOkfConceptRevision,
    pub actor: String,
}

pub struct OkfConceptService<'a> {
    drive: &'a dyn KnowledgeDriveStorage,
    object_refs: &'a dyn KnowledgeDriveObjectRefStore,
    concept_store: &'a dyn KnowledgeOkfConceptStore,
    link_store: Option<&'a dyn KnowledgeOkfConceptLinkStore>,
    candidate_store: Option<&'a dyn KnowledgeOkfCandidateStore>,
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
            link_store: None,
            candidate_store: None,
            file_entries: None,
            drive_workspace: None,
        }
    }

    pub fn with_link_store(mut self, link_store: &'a dyn KnowledgeOkfConceptLinkStore) -> Self {
        self.link_store = Some(link_store);
        self
    }

    pub fn with_candidate_store(
        mut self,
        candidate_store: &'a dyn KnowledgeOkfCandidateStore,
    ) -> Self {
        self.candidate_store = Some(candidate_store);
        self
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
        let staged = self
            .stage_concept_revision(&request, drive_space_id, true)
            .await?;
        self.finalize_publication(
            request.space_id,
            &request.concept_id,
            &request.title,
            &request.actor,
            &staged,
            drive_space_id,
        )
        .await?;
        Ok(staged.publication)
    }

    pub async fn stage_concept_candidate(
        &self,
        request: PublishKnowledgeOkfConceptRequest,
        drive_space_id: Option<&str>,
    ) -> Result<KnowledgeOkfConceptPublication, OkfConceptServiceError> {
        let staged = self
            .stage_concept_revision(&request, drive_space_id, false)
            .await?;
        Ok(staged.publication)
    }

    pub async fn upsert_concept_from_markdown(
        &self,
        request: OkfConceptUpsertRequest,
        drive_space_id: Option<&str>,
    ) -> Result<KnowledgeOkfConceptPublication, OkfConceptServiceError> {
        if request.space_id == 0 {
            return Err(OkfConceptServiceError::InvalidRequest(
                "space_id is required".to_string(),
            ));
        }
        if request.actor.trim().is_empty() {
            return Err(OkfConceptServiceError::InvalidRequest(
                "actor is required".to_string(),
            ));
        }
        if request.markdown.trim().is_empty() {
            return Err(OkfConceptServiceError::InvalidRequest(
                "markdown is required".to_string(),
            ));
        }

        let concept_id = normalize_concept_id(&request.concept_id)?;
        let document = parse_okf_markdown(&request.markdown)
            .map_err(|error| OkfConceptServiceError::InvalidRequest(error.to_string()))?
            .ok_or_else(|| {
                OkfConceptServiceError::InvalidRequest(
                    "markdown must include OKF concept frontmatter with type".to_string(),
                )
            })?;
        validate_concept_document(&document, &concept_id)
            .map_err(OkfConceptServiceError::Conformance)?;

        let title = document
            .title
            .clone()
            .filter(|value| !value.trim().is_empty())
            .unwrap_or_else(|| title_from_concept_id(&concept_id));
        let publish_request = PublishKnowledgeOkfConceptRequest {
            space_id: request.space_id,
            concept_id,
            title,
            concept_type: document.concept_type,
            description: document.description.unwrap_or_default(),
            markdown: document.body,
            source_count: 0,
            tags: document.tags,
            actor: request.actor,
        };

        let staged = self
            .stage_concept_revision(&publish_request, drive_space_id, request.publish)
            .await?;
        if request.publish {
            self.finalize_publication(
                publish_request.space_id,
                &publish_request.concept_id,
                &publish_request.title,
                &publish_request.actor,
                &staged,
                drive_space_id,
            )
            .await?;
        }
        Ok(staged.publication)
    }

    pub async fn publish_existing_revision(
        &self,
        request: PublishExistingOkfConceptRevisionRequest,
        drive_space_id: Option<&str>,
    ) -> Result<KnowledgeOkfConceptPublication, OkfConceptServiceError> {
        if request.space_id == 0 {
            return Err(OkfConceptServiceError::InvalidRequest(
                "space_id is required".to_string(),
            ));
        }
        if request.actor.trim().is_empty() {
            return Err(OkfConceptServiceError::InvalidRequest(
                "actor is required".to_string(),
            ));
        }
        if request.concept.space_id != request.space_id {
            return Err(OkfConceptServiceError::InvalidRequest(
                "concept does not belong to the requested space".to_string(),
            ));
        }
        if request.concept.publish_state == OkfConceptPublishState::Published {
            return Err(OkfConceptServiceError::InvalidRequest(
                "concept is already published".to_string(),
            ));
        }

        let governance_revision_path =
            governance_revision_path(&request.concept.concept_id, request.revision.revision_no);
        let revision_markdown = read_managed_markdown(self.drive, &governance_revision_path)
            .await
            .map_err(|error| {
                OkfConceptServiceError::Internal(format!(
                    "failed to read governance revision at {governance_revision_path}: {error}"
                ))
            })?;
        let document = parse_okf_markdown(&revision_markdown)
            .map_err(|error| OkfConceptServiceError::InvalidRequest(error.to_string()))?
            .ok_or_else(|| {
                OkfConceptServiceError::InvalidRequest(
                    "governance revision must include OKF concept frontmatter".to_string(),
                )
            })?;
        validate_concept_document(&document, &request.concept.concept_id)
            .map_err(OkfConceptServiceError::Conformance)?;

        let published_logical_path =
            OkfBundlePaths::concept_logical_path(&request.concept.concept_id);
        let published_ref = self
            .put_markdown(
                &published_logical_path,
                "concept_revision",
                &revision_markdown,
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

        let concept = self
            .concept_store
            .mark_current_revision(MarkKnowledgeOkfConceptCurrentRevisionRecord {
                concept_row_id: request.concept.id,
                revision_id: request.revision.id,
                publish_state: OkfConceptPublishState::Published,
            })
            .await?;

        let publication = KnowledgeOkfConceptPublication {
            concept,
            revision: request.revision,
            published_logical_path: published_logical_path.clone(),
            governance_revision_path,
        };

        self.ensure_drive_nodes(
            drive_space_id.as_deref(),
            vec![
                folder_node(".sdkwork/governance/revisions"),
                folder_node(&format!(
                    ".sdkwork/governance/revisions/{}",
                    request.concept.concept_id
                )),
                file_node(&published_ref),
            ],
        )
        .await?;

        self.concept_store
            .append_log_entry(AppendKnowledgeOkfLogEntryRecord {
                space_id: request.space_id,
                event_type: OkfLogEventType::Publish.as_str().to_string(),
                event_time: now_rfc3339()?,
                title: format!("Published {}", request.concept.title),
                actor: request.actor,
                affected_concepts: vec![request.concept.title.clone()],
                audit_event_id: None,
                warnings: vec![],
                privacy_level: "internal".to_string(),
            })
            .await?;
        self.reindex_concept_links(
            request.space_id,
            &request.concept.concept_id,
            &document.body,
        )
        .await?;
        self.rebuild_standard_files(request.space_id, drive_space_id.as_deref())
            .await?;

        self.update_candidate_state(
            request.concept.id,
            OkfConceptPublishState::Published,
            None,
            None,
        )
        .await?;

        Ok(publication)
    }

    async fn stage_concept_revision(
        &self,
        request: &PublishKnowledgeOkfConceptRequest,
        drive_space_id: Option<&str>,
        project_to_bundle: bool,
    ) -> Result<StagedOkfConceptRevision, OkfConceptServiceError> {
        validate_publish_request(request)?;
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
        let publish_state = if project_to_bundle {
            OkfConceptPublishState::Published
        } else {
            OkfConceptPublishState::CandidateReady
        };

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
                publish_state,
            })
            .await?;
        let revision_no = self.concept_store.next_revision_no(concept.id).await?;
        let governance_revision_path = governance_revision_path(&concept_id, revision_no);

        let published_ref = if project_to_bundle {
            Some(
                self.put_markdown(
                    &published_logical_path,
                    "concept_revision",
                    &published_markdown,
                )
                .await?,
            )
        } else {
            None
        };
        let revision_ref = self
            .put_markdown(
                &governance_revision_path,
                "concept_revision",
                &published_markdown,
            )
            .await?;
        if let Some(published_ref) = &published_ref {
            self.object_refs
                .create_or_get_object_ref(object_ref_record(
                    request.space_id,
                    drive_space_id.as_deref(),
                    None,
                    published_ref,
                ))
                .await?;
        }
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
                publish_state,
            })
            .await?;

        let mut drive_nodes = vec![
            folder_node(".sdkwork/governance/revisions"),
            folder_node(&format!(".sdkwork/governance/revisions/{concept_id}")),
            file_node(&revision_ref),
        ];
        if let Some(published_ref) = &published_ref {
            drive_nodes.push(file_node(published_ref));
        }
        self.ensure_drive_nodes(drive_space_id.as_deref(), drive_nodes)
            .await?;

        if !project_to_bundle {
            let candidate_type = if revision_no > 1 {
                OkfCandidateType::ConceptUpdate
            } else {
                OkfCandidateType::ConceptCreate
            };
            self.record_staged_candidate(
                request.space_id,
                &concept_id,
                concept.id,
                markdown_object_ref.id,
                candidate_type,
            )
            .await?;
        }

        Ok(StagedOkfConceptRevision {
            publication: KnowledgeOkfConceptPublication {
                concept,
                revision,
                published_logical_path,
                governance_revision_path,
            },
            concept_document,
        })
    }

    async fn finalize_publication(
        &self,
        space_id: u64,
        concept_id: &str,
        title: &str,
        actor: &str,
        staged: &StagedOkfConceptRevision,
        drive_space_id: Option<&str>,
    ) -> Result<(), OkfConceptServiceError> {
        self.concept_store
            .append_log_entry(AppendKnowledgeOkfLogEntryRecord {
                space_id,
                event_type: OkfLogEventType::Publish.as_str().to_string(),
                event_time: now_rfc3339()?,
                title: format!("Published {title}"),
                actor: actor.to_string(),
                affected_concepts: vec![title.to_string()],
                audit_event_id: None,
                warnings: vec![],
                privacy_level: "internal".to_string(),
            })
            .await?;
        self.reindex_concept_links(space_id, concept_id, &staged.concept_document.body)
            .await?;
        self.rebuild_standard_files(space_id, drive_space_id.as_deref())
            .await?;
        Ok(())
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
        self.ensure_drive_nodes(
            drive_space_id.as_deref(),
            [file_node(&index_ref), file_node(&log_ref)].into(),
        )
        .await?;
        Ok(())
    }

    async fn ensure_drive_nodes(
        &self,
        drive_space_id: Option<&str>,
        nodes: Vec<EnsureKnowledgeDriveNodeRequest>,
    ) -> Result<(), OkfConceptServiceError> {
        let Some(workspace) = self.drive_workspace else {
            return Ok(());
        };
        if nodes.is_empty() {
            return Ok(());
        }
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
                nodes,
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

    async fn reindex_concept_links(
        &self,
        space_id: u64,
        concept_id: &str,
        body: &str,
    ) -> Result<(), OkfConceptServiceError> {
        let Some(link_store) = self.link_store else {
            return Ok(());
        };
        let known = self
            .concept_store
            .list_concept_summaries(space_id)
            .await?
            .into_iter()
            .map(|summary| summary.concept_id)
            .collect::<Vec<_>>();
        let links = index_concept_links(body, concept_id, &known)
            .into_iter()
            .filter_map(|link| {
                link.target_concept_id
                    .map(|to_concept_id| KnowledgeOkfConceptLinkRecord {
                        to_concept_id,
                        anchor_text: link.anchor_text,
                    })
            })
            .collect();
        link_store
            .replace_outbound_links(ReplaceKnowledgeOkfConceptLinksRecord {
                space_id,
                from_concept_id: concept_id.to_string(),
                links,
            })
            .await?;
        Ok(())
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

impl<'a> OkfConceptService<'a> {
    async fn record_staged_candidate(
        &self,
        space_id: u64,
        concept_id: &str,
        concept_row_id: u64,
        markdown_object_ref_id: u64,
        candidate_type: OkfCandidateType,
    ) -> Result<(), OkfConceptServiceError> {
        let Some(candidate_store) = self.candidate_store else {
            return Ok(());
        };
        candidate_store
            .upsert_candidate(UpsertKnowledgeOkfCandidateRecord {
                space_id,
                concept_row_id,
                concept_id: concept_id.to_string(),
                candidate_type,
                state: OkfConceptPublishState::CandidateReady,
                markdown_object_ref_id,
            })
            .await
            .map_err(OkfConceptServiceError::CandidateStore)
    }

    async fn update_candidate_state(
        &self,
        concept_row_id: u64,
        state: OkfConceptPublishState,
        reviewer_id: Option<u64>,
        review_note: Option<String>,
    ) -> Result<(), OkfConceptServiceError> {
        let Some(candidate_store) = self.candidate_store else {
            return Ok(());
        };
        candidate_store
            .update_candidate_state_by_concept_row_id(
                concept_row_id,
                state,
                reviewer_id,
                review_note,
            )
            .await
            .map_err(OkfConceptServiceError::CandidateStore)
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

#[derive(Debug, Clone)]
struct StagedOkfConceptRevision {
    publication: KnowledgeOkfConceptPublication,
    concept_document: OkfConceptDocument,
}

fn governance_revision_path(concept_id: &str, revision_no: u64) -> String {
    format!(".sdkwork/governance/revisions/{concept_id}/r{revision_no}.md")
}

fn title_from_concept_id(concept_id: &str) -> String {
    concept_id
        .rsplit('/')
        .next()
        .unwrap_or(concept_id)
        .replace(['-', '_'], " ")
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
    LinkStore(#[from] KnowledgeOkfConceptLinkStoreError),
    #[error(transparent)]
    CandidateStore(#[from] KnowledgeOkfCandidateStoreError),
    #[error(transparent)]
    FileEntryStore(#[from] KnowledgeOkfBundleFileStoreError),
    #[error(transparent)]
    DriveWorkspace(#[from] KnowledgeDriveWorkspaceError),
    #[error("knowledge okf concept internal error: {0}")]
    Internal(String),
}
