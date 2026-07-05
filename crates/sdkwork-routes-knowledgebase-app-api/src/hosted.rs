use async_trait::async_trait;
use sdkwork_intelligence_knowledgebase_service::ports::knowledge_drive_node_tree::{
    GetKnowledgeDriveNodeRequest, KnowledgeDriveNodeTree,
};
use sdkwork_intelligence_knowledgebase_service::ports::knowledge_drive_storage::{
    KnowledgeDriveStorage, PutKnowledgeObjectRequest,
};
use sdkwork_intelligence_knowledgebase_service::{
    browser::{KnowledgeBrowserAccessContext, KnowledgeBrowserService},
    imports::KnowledgeDriveImportService,
    imports::KnowledgeGitImportService,
    imports::{KnowledgeDocumentMarkdownReader, KnowledgeGitSyncService},
    ingest::ApiMarkdownIngestPipeline,
    ports::{
        knowledge_document_store::{
            CreateKnowledgeDocumentRecord, KnowledgeDocumentIdentityScope, KnowledgeDocumentStore,
        },
        knowledge_document_version_store::{
            CreateKnowledgeDocumentVersionRecord, KnowledgeDocumentVersionStore,
        },
        knowledge_okf_bundle_file_store::{
            CreateKnowledgeOkfBundleFileRecord, KnowledgeOkfBundleFileStore,
        },
        knowledge_okf_concept_store::KnowledgeOkfConceptStore,
        knowledge_space_store::KnowledgeSpaceStore,
    },
};
use sdkwork_knowledgebase_contract::{
    CreateKnowledgeDocumentRequest, CreateKnowledgeDocumentVersionRequest,
    CreateKnowledgeSpaceRequest, GrantKnowledgeSpaceMemberRequest, IngestionJob, KnowledgeDocument,
    KnowledgeDocumentContent, KnowledgeDocumentVersion, KnowledgeDocumentVisibility,
    KnowledgeDriveImportRequest, KnowledgeDriveImportResult, KnowledgeGitImportRequest,
    KnowledgeGitImportResult, KnowledgeGitSyncRequest, KnowledgeGitSyncResult,
    KnowledgeIngestRequest, KnowledgeOkfBundleFile, KnowledgeOkfConceptRevisionList,
    KnowledgeSpace, KnowledgeSpaceMember, KnowledgeSpaceMemberSubjectType,
    ListKnowledgeBrowserRequest, OkfBundleExportRequest, OkfBundleFileKind, OkfBundleImportRequest,
    OkfBundleImportResult, OkfConceptSummary, OkfConceptUpsertRequest, OkfContextPackRequest,
    OkfFileAnswerRequest, OkfIndexDocument, OkfLogDocument, OkfProfileDocument, OkfQualityRun,
    OkfQualityRunRequest, OkfQueryRequest, OkfQueryResult, PublishKnowledgeOkfConceptRequest,
    UpdateKnowledgeSpaceRequest,
};
use sdkwork_utils_rust::{is_blank, SdkWorkPageData};

use crate::{
    hosted_access::{
        create_space_with_context, delete_space_with_context, ensure_runtime_tenant,
        grant_space_member_with_context, list_space_members_with_context, require_document_access,
        require_ingest_access, require_okf_concept_space_access, require_space_access,
        revoke_space_member_with_context, update_space_with_context,
    },
    hosted_support::{
        build_okf_context_pack_from_engine, concept_to_summary, create_okf_bundle_export,
        create_okf_bundle_import, create_okf_lint_run, format_okf_engine_answer,
        okf_answer_concept_id, okf_paths, read_managed_okf_text, retrieve_okf_bundle_export,
        stable_u64_hash,
    },
    runtime::KnowledgebaseRuntime,
    ApiError, ApiResult, KnowledgeAppRequestContext, KnowledgeBrowserApi,
    KnowledgeDocumentAppService, KnowledgeDriveImportAppService, KnowledgeGitImportAppService,
    KnowledgeIngestAppService, KnowledgeOkfAppService, KnowledgeSpaceAppService,
};

#[derive(Clone)]
pub(crate) struct HostedSpaceService {
    runtime: KnowledgebaseRuntime,
}

impl HostedSpaceService {
    pub fn new(runtime: KnowledgebaseRuntime) -> Self {
        Self { runtime }
    }
}

#[async_trait]
impl KnowledgeSpaceAppService for HostedSpaceService {
    async fn create_space(
        &self,
        context: KnowledgeAppRequestContext,
        mut request: CreateKnowledgeSpaceRequest,
    ) -> ApiResult<KnowledgeSpace> {
        if request.owner_subject_id.is_none() {
            if let Some(actor_id) = context.actor_id {
                request.owner_subject_id = Some(actor_id.to_string());
            }
        }
        if request.owner_subject_type.is_none() {
            request.owner_subject_type = Some("user".to_string());
        }
        create_space_with_context(&self.runtime, &context, request).await
    }

    async fn retrieve_space(
        &self,
        context: KnowledgeAppRequestContext,
        space_id: u64,
    ) -> ApiResult<KnowledgeSpace> {
        require_space_access(&self.runtime, &context, space_id).await
    }

    async fn update_space(
        &self,
        context: KnowledgeAppRequestContext,
        space_id: u64,
        request: UpdateKnowledgeSpaceRequest,
    ) -> ApiResult<KnowledgeSpace> {
        update_space_with_context(&self.runtime, &context, space_id, request).await
    }

    async fn delete_space(
        &self,
        context: KnowledgeAppRequestContext,
        space_id: u64,
    ) -> ApiResult<()> {
        delete_space_with_context(&self.runtime, &context, space_id).await
    }

    async fn list_space_members(
        &self,
        context: KnowledgeAppRequestContext,
        space_id: u64,
        cursor: Option<String>,
        page_size: Option<u32>,
    ) -> ApiResult<SdkWorkPageData<KnowledgeSpaceMember>> {
        list_space_members_with_context(&self.runtime, &context, space_id, cursor, page_size).await
    }

    async fn grant_space_member(
        &self,
        context: KnowledgeAppRequestContext,
        space_id: u64,
        request: GrantKnowledgeSpaceMemberRequest,
    ) -> ApiResult<()> {
        grant_space_member_with_context(&self.runtime, &context, space_id, request).await
    }

    async fn revoke_space_member(
        &self,
        context: KnowledgeAppRequestContext,
        space_id: u64,
        subject_type: KnowledgeSpaceMemberSubjectType,
        subject_id: String,
    ) -> ApiResult<()> {
        revoke_space_member_with_context(
            &self.runtime,
            &context,
            space_id,
            subject_type,
            &subject_id,
        )
        .await
    }
}

#[derive(Clone)]
pub(crate) struct HostedIngestService {
    runtime: KnowledgebaseRuntime,
}

impl HostedIngestService {
    pub fn new(runtime: KnowledgebaseRuntime) -> Self {
        Self { runtime }
    }
}

#[async_trait]
impl KnowledgeIngestAppService for HostedIngestService {
    async fn create_ingest(
        &self,
        context: KnowledgeAppRequestContext,
        request: KnowledgeIngestRequest,
    ) -> ApiResult<IngestionJob> {
        let space_id = request.space_id;
        let space = require_space_access(&self.runtime, &context, space_id).await?;
        crate::tenant_quota_enforcement::ensure_tenant_can_start_ingest(&self.runtime).await?;
        crate::tenant_quota_enforcement::ensure_tenant_can_add_storage(
            &self.runtime,
            u64::try_from(request.payload_markdown.len()).unwrap_or(u64::MAX),
        )
        .await?;
        let pipeline = ApiMarkdownIngestPipeline::new(
            self.runtime.drive_storage(),
            self.runtime.ingestion_job_store(),
            self.runtime.markdown_index_metadata_store(),
        );
        let result = pipeline
            .run(request, space.drive_space_id.as_deref(), "api-ingest")
            .await
            .map_err(ApiError::from)?;
        if let Some(document_version_id) = result.document_version_id {
            let _ = self
                .runtime
                .try_embed_document_version(space_id, document_version_id)
                .await;
        }
        Ok(result.job)
    }

    async fn retrieve_ingest(
        &self,
        context: KnowledgeAppRequestContext,
        ingest_id: u64,
    ) -> ApiResult<IngestionJob> {
        require_ingest_access(&self.runtime, &context, ingest_id).await
    }
}

#[derive(Clone)]
pub(crate) struct HostedDriveImportService {
    runtime: KnowledgebaseRuntime,
}

impl HostedDriveImportService {
    pub fn new(runtime: KnowledgebaseRuntime) -> Self {
        Self { runtime }
    }
}

#[async_trait]
impl KnowledgeDriveImportAppService for HostedDriveImportService {
    async fn import_drive_object(
        &self,
        context: KnowledgeAppRequestContext,
        request: KnowledgeDriveImportRequest,
    ) -> ApiResult<KnowledgeDriveImportResult> {
        require_space_access(&self.runtime, &context, request.space_id).await?;
        let request = resolve_drive_import_request(self.runtime.drive_tree(), request).await?;
        crate::tenant_quota_enforcement::ensure_tenant_can_start_ingest(&self.runtime).await?;
        crate::tenant_quota_enforcement::ensure_tenant_can_import_drive_object(
            &self.runtime,
            &request,
        )
        .await?;
        let service = KnowledgeDriveImportService::new(
            self.runtime.drive_storage(),
            self.runtime.drive_import_metadata_store(),
        );
        let result = service
            .import_drive_object(request)
            .await
            .map_err(ApiError::from)?;

        let pipeline = sdkwork_intelligence_knowledgebase_service::ingest::KnowledgeIngestionJobWorkerService::new(
            self.runtime.ingestion_job_store(),
            self.runtime.drive_storage(),
        );
        match pipeline.process_drive_import_result(&result).await {
            Ok(pipeline_result) => {
                if let Some(index_result) = pipeline_result.index_result {
                    let _ = self
                        .runtime
                        .try_embed_document_version(
                            result.document.space_id,
                            index_result.document_version_id,
                        )
                        .await;
                }
                Ok(KnowledgeDriveImportResult {
                    job: pipeline_result.job,
                    ..result
                })
            }
            Err(error) => Err(ApiError::from(error)),
        }
    }
}

#[derive(Clone)]
pub(crate) struct HostedGitImportService {
    runtime: KnowledgebaseRuntime,
}

impl HostedGitImportService {
    pub fn new(runtime: KnowledgebaseRuntime) -> Self {
        Self { runtime }
    }
}

#[async_trait]
impl KnowledgeGitImportAppService for HostedGitImportService {
    async fn create_git_import(
        &self,
        context: KnowledgeAppRequestContext,
        request: KnowledgeGitImportRequest,
    ) -> ApiResult<KnowledgeGitImportResult> {
        let space_id = request.space_id;
        let space = require_space_access(&self.runtime, &context, space_id).await?;
        crate::tenant_quota_enforcement::ensure_tenant_can_start_ingest(&self.runtime).await?;
        let service = KnowledgeGitImportService::new(
            self.runtime.drive_storage(),
            self.runtime.ingestion_job_store(),
            self.runtime.markdown_index_metadata_store(),
        );
        let run = service
            .import_repository(request, space.drive_space_id.as_deref())
            .await
            .map_err(ApiError::from)?;
        for document_version_id in run.document_version_ids {
            let _ = self
                .runtime
                .try_embed_document_version(space_id, document_version_id)
                .await;
        }
        Ok(run.result)
    }

    async fn create_git_sync(
        &self,
        context: KnowledgeAppRequestContext,
        request: KnowledgeGitSyncRequest,
    ) -> ApiResult<KnowledgeGitSyncResult> {
        let space_id = request.space_id;
        require_space_access(&self.runtime, &context, space_id).await?;
        let markdown_reader = RuntimeDocumentMarkdownReader::new(self.runtime.clone());
        let service = KnowledgeGitSyncService::new(self.runtime.document_store(), &markdown_reader);
        service
            .sync_repository(request)
            .await
            .map_err(ApiError::from)
    }
}

#[derive(Clone)]
pub(crate) struct RuntimeDocumentMarkdownReader {
    runtime: crate::runtime::KnowledgebaseRuntime,
}

impl RuntimeDocumentMarkdownReader {
    pub(crate) fn new(runtime: crate::runtime::KnowledgebaseRuntime) -> Self {
        Self { runtime }
    }
}

#[async_trait]
impl KnowledgeDocumentMarkdownReader for RuntimeDocumentMarkdownReader {
    async fn read_document_markdown(&self, document_id: u64) -> Result<String, String> {
        self.runtime
            .read_document_content_markdown(document_id)
            .await
            .map(|content| content.content_markdown)
    }
}

#[derive(Clone)]
pub(crate) struct HostedDocumentService {
    runtime: KnowledgebaseRuntime,
}

impl HostedDocumentService {
    pub fn new(runtime: KnowledgebaseRuntime) -> Self {
        Self { runtime }
    }
}

#[async_trait]
impl KnowledgeDocumentAppService for HostedDocumentService {
    async fn list_documents(
        &self,
        context: KnowledgeAppRequestContext,
        space_id: u64,
        cursor: Option<String>,
        page_size: Option<u32>,
    ) -> ApiResult<SdkWorkPageData<KnowledgeDocument>> {
        if space_id == 0 {
            return Err(ApiError::invalid_request(
                "invalid_knowledge_document_list_request",
                "space_id is required",
            ));
        }
        require_space_access(&self.runtime, &context, space_id).await?;
        let normalized_page_size = crate::pagination::normalize_page_size(page_size);
        let cursor_id = crate::pagination::parse_u64_cursor(cursor.as_deref()).map_err(|_| {
            ApiError::invalid_request("invalid_parameter", "cursor must be a valid document id")
        })?;
        let (items, next_cursor, has_more) = self
            .runtime
            .document_store()
            .list_documents_page(space_id, cursor_id, normalized_page_size)
            .await
            .map_err(ApiError::from)?;
        Ok(crate::pagination::cursor_page_data(
            items,
            next_cursor,
            has_more,
            normalized_page_size,
        ))
    }

    async fn create_document(
        &self,
        context: KnowledgeAppRequestContext,
        request: CreateKnowledgeDocumentRequest,
    ) -> ApiResult<KnowledgeDocument> {
        if is_blank(Some(request.title.as_str())) {
            return Err(ApiError::invalid_request(
                "invalid_knowledge_document_request",
                "title is required",
            ));
        }
        if request.space_id == 0 {
            return Err(ApiError::invalid_request(
                "invalid_knowledge_document_request",
                "space_id is required",
            ));
        }
        require_space_access(&self.runtime, &context, request.space_id).await?;
        crate::tenant_quota_enforcement::ensure_tenant_can_create_document(&self.runtime).await?;

        self.runtime
            .document_store()
            .create_document(CreateKnowledgeDocumentRecord {
                space_id: request.space_id,
                collection_id: request.collection_id.unwrap_or(0),
                source_id: request.source_id,
                identity_scope: KnowledgeDocumentIdentityScope::SourceOnly,
                original_file_drive_node_id: None,
                title: request.title,
                mime_type: request.mime_type,
                language: request.language,
            })
            .await
            .map_err(Into::into)
    }

    async fn retrieve_document(
        &self,
        context: KnowledgeAppRequestContext,
        document_id: u64,
    ) -> ApiResult<KnowledgeDocument> {
        require_document_access(&self.runtime, &context, document_id).await
    }

    async fn update_document(
        &self,
        context: KnowledgeAppRequestContext,
        document_id: u64,
        request: CreateKnowledgeDocumentRequest,
    ) -> ApiResult<KnowledgeDocument> {
        let document = require_document_access(&self.runtime, &context, document_id).await?;
        if is_blank(Some(request.title.as_str())) {
            return Err(ApiError::invalid_request(
                "invalid_knowledge_document_request",
                "title is required",
            ));
        }
        if request.space_id != 0 && request.space_id != document.space_id {
            return Err(ApiError::invalid_request(
                "invalid_knowledge_document_request",
                "space_id does not match the document space",
            ));
        }
        let previous_visibility = document.visibility;
        let updated = self
            .runtime
            .document_store()
            .update_document_metadata(
                document_id,
                request.title,
                request.mime_type,
                request.language,
                request.visibility,
            )
            .await
            .map_err(ApiError::from)?;
        if let Some(new_visibility) = request.visibility {
            if new_visibility != previous_visibility {
                sdkwork_knowledgebase_observability::audit::record_document_visibility_changed(
                    document_id,
                    document.space_id,
                    context.actor_id.unwrap_or(0),
                    document_visibility_label(previous_visibility),
                    document_visibility_label(new_visibility),
                );
            }
        }
        Ok(updated)
    }

    async fn delete_document(
        &self,
        context: KnowledgeAppRequestContext,
        document_id: u64,
    ) -> ApiResult<()> {
        require_document_access(&self.runtime, &context, document_id).await?;
        self.runtime
            .document_store()
            .soft_delete_document(document_id)
            .await
            .map_err(Into::into)
    }

    async fn list_document_versions(
        &self,
        context: KnowledgeAppRequestContext,
        document_id: u64,
        cursor: Option<String>,
        page_size: Option<u32>,
    ) -> ApiResult<SdkWorkPageData<KnowledgeDocumentVersion>> {
        require_document_access(&self.runtime, &context, document_id).await?;
        let normalized_page_size = crate::pagination::normalize_page_size(page_size);
        let cursor_id = crate::pagination::parse_u64_cursor(cursor.as_deref()).map_err(|_| {
            ApiError::invalid_request(
                "invalid_parameter",
                "cursor must be a valid document version id",
            )
        })?;
        let (items, next_cursor, has_more) = self
            .runtime
            .version_store()
            .list_versions_page_for_document(document_id, cursor_id, normalized_page_size)
            .await
            .map_err(map_document_version_error)?;
        Ok(crate::pagination::cursor_page_data(
            items,
            next_cursor,
            has_more,
            normalized_page_size,
        ))
    }

    async fn create_document_version(
        &self,
        context: KnowledgeAppRequestContext,
        document_id: u64,
        request: CreateKnowledgeDocumentVersionRequest,
    ) -> ApiResult<KnowledgeDocumentVersion> {
        require_document_access(&self.runtime, &context, document_id).await?;
        if request.document_id != 0 && request.document_id != document_id {
            return Err(ApiError::invalid_request(
                "invalid_knowledge_document_version_request",
                "document_id in body must match path documentId when provided",
            ));
        }
        self.runtime
            .document_store()
            .get_document_by_id(document_id)
            .await
            .map_err(ApiError::from)?;

        let existing = self
            .runtime
            .version_store()
            .list_versions_for_document(document_id)
            .await
            .map_err(map_document_version_error)?;
        let version_no = existing
            .iter()
            .map(|version| version.version_no)
            .max()
            .unwrap_or(0)
            .saturating_add(1);

        self.runtime
            .version_store()
            .create_document_version(CreateKnowledgeDocumentVersionRecord {
                document_id,
                version_no,
                original_object_ref_id: request.original_object_ref_id,
                checksum_sha256_hex: request.checksum_sha256_hex,
                size_bytes: request.size_bytes,
                mime_type: request.mime_type,
            })
            .await
            .map_err(map_document_version_error)
    }

    async fn retrieve_document_content(
        &self,
        context: KnowledgeAppRequestContext,
        document_id: u64,
    ) -> ApiResult<KnowledgeDocumentContent> {
        require_document_access(&self.runtime, &context, document_id).await?;
        self.runtime
            .read_document_content_markdown(document_id)
            .await
            .map_err(|detail| ApiError::internal("document_content_read_failed", detail))
    }
}

#[derive(Clone)]
pub(crate) struct HostedBrowserService {
    runtime: KnowledgebaseRuntime,
}

impl HostedBrowserService {
    pub fn new(runtime: KnowledgebaseRuntime) -> Self {
        Self { runtime }
    }
}

#[async_trait]
impl KnowledgeBrowserApi for HostedBrowserService {
    async fn list_browser(
        &self,
        context: KnowledgeAppRequestContext,
        request: ListKnowledgeBrowserRequest,
    ) -> ApiResult<SdkWorkPageData<sdkwork_knowledgebase_contract::KnowledgeBrowserNode>> {
        ensure_runtime_tenant(&self.runtime, &context)?;
        let actor_id = context
            .actor_id
            .map(|value| value.to_string())
            .unwrap_or_else(|| "anonymous".to_string());
        let service = KnowledgeBrowserService::new(
            self.runtime.space_store(),
            self.runtime.drive_tree(),
            self.runtime.browser_projection_store(),
        )
        .with_access_control(self.runtime.access_control());
        let page = service
            .list(
                Some(KnowledgeBrowserAccessContext {
                    tenant_id: context.tenant_id,
                    actor_id,
                }),
                request,
            )
            .await
            .map_err(ApiError::from)?;
        Ok(crate::pagination::browser_list_page_data(
            page.items,
            page.next_cursor,
            page.page_size,
        ))
    }
}

#[derive(Clone)]
pub(crate) struct HostedOkfService {
    runtime: KnowledgebaseRuntime,
}

impl HostedOkfService {
    pub fn new(runtime: KnowledgebaseRuntime) -> Self {
        Self { runtime }
    }
}

#[async_trait]
impl KnowledgeOkfAppService for HostedOkfService {
    async fn list_okf_concepts(
        &self,
        context: KnowledgeAppRequestContext,
        space_id: u64,
        cursor: Option<String>,
        page_size: Option<u32>,
    ) -> ApiResult<SdkWorkPageData<OkfConceptSummary>> {
        if space_id == 0 {
            return Err(ApiError::invalid_request(
                "invalid_okf_concept_list_request",
                "space_id is required",
            ));
        }
        require_space_access(&self.runtime, &context, space_id).await?;
        self.runtime
            .resolve_okf_bundle_engine_for_space(space_id)
            .await?;
        let normalized_page_size = crate::pagination::normalize_page_size(page_size);
        let cursor_id = crate::pagination::parse_u64_cursor(cursor.as_deref()).map_err(|_| {
            ApiError::invalid_request("invalid_parameter", "cursor must be a valid concept id")
        })?;
        let (items, next_cursor, has_more) = self
            .runtime
            .okf_concept_store()
            .list_concept_summaries_page(space_id, cursor_id, normalized_page_size)
            .await
            .map_err(map_okf_concept_store_error)?;
        Ok(crate::pagination::cursor_page_data(
            items,
            next_cursor,
            has_more,
            normalized_page_size,
        ))
    }

    async fn retrieve_okf_concept(
        &self,
        context: KnowledgeAppRequestContext,
        concept_row_id: u64,
    ) -> ApiResult<OkfConceptSummary> {
        require_okf_concept_space_access(&self.runtime, &context, concept_row_id).await?;
        let page = self
            .runtime
            .okf_concept_store()
            .get_concept_by_row_id(concept_row_id)
            .await
            .map_err(map_okf_concept_store_error)?;
        Ok(concept_to_summary(page))
    }

    async fn list_okf_concept_revisions(
        &self,
        context: KnowledgeAppRequestContext,
        concept_row_id: u64,
    ) -> ApiResult<KnowledgeOkfConceptRevisionList> {
        require_okf_concept_space_access(&self.runtime, &context, concept_row_id).await?;
        let items = self
            .runtime
            .okf_concept_store()
            .list_concept_revisions(concept_row_id)
            .await
            .map_err(map_okf_concept_store_error)?;
        Ok(KnowledgeOkfConceptRevisionList { items })
    }

    async fn upsert_okf_concept(
        &self,
        context: KnowledgeAppRequestContext,
        request: OkfConceptUpsertRequest,
    ) -> ApiResult<OkfConceptSummary> {
        if request.space_id == 0 {
            return Err(ApiError::invalid_request(
                "invalid_okf_concept_upsert_request",
                "space_id is required",
            ));
        }
        if is_blank(Some(request.concept_id.as_str())) {
            return Err(ApiError::invalid_request(
                "invalid_okf_concept_upsert_request",
                "concept_id is required",
            ));
        }
        if is_blank(Some(request.markdown.as_str())) {
            return Err(ApiError::invalid_request(
                "invalid_okf_concept_upsert_request",
                "markdown is required",
            ));
        }

        require_space_access(&self.runtime, &context, request.space_id).await?;
        let actor = context
            .actor_id
            .map(|value| value.to_string())
            .filter(|value| !is_blank(Some(value.as_str())))
            .unwrap_or_else(|| {
                if is_blank(Some(request.actor.as_str())) {
                    self.runtime.operator_id().to_string()
                } else {
                    request.actor.clone()
                }
            });
        self.runtime
            .resolve_okf_bundle_engine_for_space(request.space_id)
            .await?;
        let concept = self
            .runtime
            .knowledge_engines()
            .upsert_okf_concept(OkfConceptUpsertRequest {
                space_id: request.space_id,
                concept_id: request.concept_id,
                markdown: request.markdown,
                actor,
                publish: request.publish,
            })
            .await
            .map_err(ApiError::from)?;
        Ok(concept_to_summary(concept))
    }

    async fn delete_okf_concept(
        &self,
        context: KnowledgeAppRequestContext,
        concept_row_id: u64,
    ) -> ApiResult<()> {
        let space =
            require_okf_concept_space_access(&self.runtime, &context, concept_row_id).await?;
        let actor = context
            .actor_id
            .map(|value| value.to_string())
            .unwrap_or_else(|| self.runtime.operator_id().to_string());
        self.runtime
            .resolve_okf_bundle_engine_for_space(space.id)
            .await?;
        self.runtime
            .knowledge_engines()
            .delete_okf_concept(space.id, concept_row_id, &actor)
            .await
            .map_err(ApiError::from)?;
        Ok(())
    }

    async fn retrieve_okf_index(
        &self,
        context: KnowledgeAppRequestContext,
        space_id: u64,
    ) -> ApiResult<OkfIndexDocument> {
        if space_id == 0 {
            return Err(ApiError::invalid_request(
                "invalid_okf_index_request",
                "space_id is required",
            ));
        }
        let space = require_space_access(&self.runtime, &context, space_id).await?;
        self.runtime
            .resolve_okf_bundle_engine_for_space(space.id)
            .await?;
        let paths = okf_paths();
        let markdown = read_managed_okf_text(
            self.runtime.drive_storage(),
            paths.index_md,
            "bundle_index",
            space.drive_space_id.as_deref(),
        )
        .await?;
        Ok(OkfIndexDocument { markdown })
    }

    async fn retrieve_okf_log(
        &self,
        context: KnowledgeAppRequestContext,
        space_id: u64,
    ) -> ApiResult<OkfLogDocument> {
        if space_id == 0 {
            return Err(ApiError::invalid_request(
                "invalid_okf_log_request",
                "space_id is required",
            ));
        }
        let space = require_space_access(&self.runtime, &context, space_id).await?;
        self.runtime
            .resolve_okf_bundle_engine_for_space(space.id)
            .await?;
        let paths = okf_paths();
        let markdown = read_managed_okf_text(
            self.runtime.drive_storage(),
            paths.log_md,
            "bundle_log",
            space.drive_space_id.as_deref(),
        )
        .await?;
        Ok(OkfLogDocument { markdown })
    }

    async fn retrieve_okf_schema(
        &self,
        context: KnowledgeAppRequestContext,
        space_id: u64,
    ) -> ApiResult<OkfProfileDocument> {
        if space_id == 0 {
            return Err(ApiError::invalid_request(
                "invalid_okf_schema_request",
                "space_id is required",
            ));
        }
        let space = require_space_access(&self.runtime, &context, space_id).await?;
        self.runtime
            .resolve_okf_bundle_engine_for_space(space.id)
            .await?;
        let paths = okf_paths();
        let agents_markdown = read_managed_okf_text(
            self.runtime.drive_storage(),
            paths.agents_md,
            "bundle_profile",
            space.drive_space_id.as_deref(),
        )
        .await?;
        let profile_yaml = read_managed_okf_text(
            self.runtime.drive_storage(),
            paths.profile_yaml,
            "bundle_profile",
            space.drive_space_id.as_deref(),
        )
        .await?;
        Ok(OkfProfileDocument {
            agents_markdown,
            profile_yaml,
        })
    }

    async fn create_okf_query(
        &self,
        context: KnowledgeAppRequestContext,
        request: OkfQueryRequest,
    ) -> ApiResult<OkfQueryResult> {
        if request.space_id == 0 {
            return Err(ApiError::invalid_request(
                "invalid_okf_query_request",
                "space_id is required",
            ));
        }
        if is_blank(Some(request.query.as_str())) {
            return Err(ApiError::invalid_request(
                "invalid_okf_query_request",
                "query is required",
            ));
        }

        require_space_access(&self.runtime, &context, request.space_id).await?;
        let search = self
            .runtime
            .search_knowledge_engine_for_space(request.space_id, &request.query, 8)
            .await
            .map_err(|error| ApiError::internal("okf_engine_search_failed", error))?;

        Ok(OkfQueryResult {
            answer_markdown: format_okf_engine_answer(&search.hits),
            trace_id: Some(format!(
                "{}:{}",
                search.implementation_id,
                stable_u64_hash(&request.query)
            )),
        })
    }

    async fn file_okf_query_answer(
        &self,
        context: KnowledgeAppRequestContext,
        query_id: u64,
        request: OkfFileAnswerRequest,
    ) -> ApiResult<OkfQueryResult> {
        if is_blank(Some(request.title.as_str())) {
            return Err(ApiError::invalid_request(
                "invalid_okf_file_answer_request",
                "title is required",
            ));
        }
        if is_blank(Some(request.answer_markdown.as_str())) {
            return Err(ApiError::invalid_request(
                "invalid_okf_file_answer_request",
                "answer_markdown is required",
            ));
        }
        if request.space_id == 0 {
            return Err(ApiError::invalid_request(
                "invalid_okf_file_answer_request",
                "space_id is required",
            ));
        }

        require_space_access(&self.runtime, &context, request.space_id).await?;
        self.runtime
            .resolve_okf_bundle_engine_for_space(request.space_id)
            .await?;
        let concept_id = okf_answer_concept_id(query_id);
        let description = request.title.clone();
        let actor = context
            .actor_id
            .map(|value| value.to_string())
            .unwrap_or_else(|| self.runtime.operator_id().to_string());
        let publication = self
            .runtime
            .knowledge_engines()
            .publish_okf_concept(PublishKnowledgeOkfConceptRequest {
                space_id: request.space_id,
                concept_id,
                title: request.title,
                concept_type: "Answer".to_string(),
                description,
                markdown: request.answer_markdown.clone(),
                source_count: 1,
                tags: vec!["okf-query".to_string()],
                actor,
                resource: None,
                timestamp: None,
            })
            .await
            .map_err(ApiError::from)?;

        Ok(OkfQueryResult {
            answer_markdown: request.answer_markdown,
            trace_id: Some(publication.concept.id.to_string()),
        })
    }

    async fn create_okf_context_pack(
        &self,
        context: KnowledgeAppRequestContext,
        request: OkfContextPackRequest,
    ) -> ApiResult<KnowledgeOkfBundleFile> {
        if request.space_id == 0 {
            return Err(ApiError::invalid_request(
                "invalid_okf_context_pack_request",
                "space_id is required",
            ));
        }

        require_space_access(&self.runtime, &context, request.space_id).await?;
        let query = request.query.unwrap_or_default();
        let context_pack =
            build_okf_context_pack_from_engine(&self.runtime, request.space_id, query, 4096)
                .await?;

        let body = serde_json::to_vec_pretty(&context_pack).map_err(|error| {
            ApiError::internal("okf_context_pack_serialization_failed", error.to_string())
        })?;
        let logical_path = format!("context_packs/cp-{}.json", context_pack.context_pack_id);
        let space = self
            .runtime
            .space_store()
            .get_space(request.space_id)
            .await?;
        let object_ref = self
            .runtime
            .drive_storage()
            .put_object(
                PutKnowledgeObjectRequest {
                    logical_path: logical_path.clone(),
                    object_role: "context_pack".to_string(),
                    content_type: "application/json; charset=utf-8".to_string(),
                    body,
                    checksum_sha256_hex: None,
                    space_uuid: None,
                }
                .with_drive_space_id(space.drive_space_id.as_deref()),
            )
            .await?;

        self.runtime
            .okf_bundle_file_store()
            .create_file_entry(CreateKnowledgeOkfBundleFileRecord {
                space_id: request.space_id,
                logical_path,
                file_kind: OkfBundleFileKind::ContextPack,
                artifact_role: object_ref.object_role.clone(),
                drive_bucket: object_ref.bucket.clone(),
                drive_object_key: object_ref.object_key.clone(),
                checksum_sha256_hex: object_ref.checksum_sha256_hex.clone(),
            })
            .await
            .map_err(map_okf_bundle_file_error)
    }

    async fn create_okf_export(
        &self,
        context: KnowledgeAppRequestContext,
        request: OkfBundleExportRequest,
    ) -> ApiResult<KnowledgeOkfBundleFile> {
        require_space_access(&self.runtime, &context, request.space_id).await?;
        create_okf_bundle_export(&self.runtime, request).await
    }

    async fn retrieve_okf_export(
        &self,
        context: KnowledgeAppRequestContext,
        export_id: u64,
    ) -> ApiResult<KnowledgeOkfBundleFile> {
        let export = retrieve_okf_bundle_export(&self.runtime, export_id).await?;
        require_space_access(&self.runtime, &context, export.space_id).await?;
        Ok(export)
    }

    async fn create_okf_import(
        &self,
        context: KnowledgeAppRequestContext,
        request: OkfBundleImportRequest,
    ) -> ApiResult<OkfBundleImportResult> {
        require_space_access(&self.runtime, &context, request.space_id).await?;
        let actor = context
            .actor_id
            .map(|value| value.to_string())
            .unwrap_or_else(|| self.runtime.operator_id().to_string());
        create_okf_bundle_import(&self.runtime, request, &actor).await
    }

    async fn create_okf_lint_run(
        &self,
        context: KnowledgeAppRequestContext,
        request: OkfQualityRunRequest,
    ) -> ApiResult<OkfQualityRun> {
        require_space_access(&self.runtime, &context, request.space_id).await?;
        create_okf_lint_run(&self.runtime, request).await
    }
}

fn map_document_version_error(
    error: sdkwork_intelligence_knowledgebase_service::ports::knowledge_document_version_store::KnowledgeDocumentVersionStoreError,
) -> ApiError {
    ApiError::internal("knowledge_document_version_store_failed", error.to_string())
}

async fn resolve_drive_import_request(
    drive_tree: &dyn KnowledgeDriveNodeTree,
    mut request: KnowledgeDriveImportRequest,
) -> ApiResult<KnowledgeDriveImportRequest> {
    let needs_locator = is_blank(Some(request.drive_object_key.as_str()))
        || is_blank(Some(request.drive_bucket.as_str()))
        || is_blank(Some(request.drive_storage_provider_id.as_str()));
    if !needs_locator {
        return Ok(request);
    }

    let Some(drive_node_id) = request
        .drive_node_id
        .as_ref()
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())
        .map(str::to_string)
    else {
        return Err(ApiError::invalid_request(
            "invalid_drive_import_request",
            "drive_storage_provider_id, drive_bucket, and drive_object_key are required when drive_node_id is absent".to_string(),
        ));
    };
    let drive_space_id = request.drive_space_id.clone().ok_or_else(|| {
        ApiError::invalid_request(
            "invalid_drive_import_request",
            "drive_space_id is required when resolving drive object locator from drive_node_id"
                .to_string(),
        )
    })?;

    let node = drive_tree
        .get_node(GetKnowledgeDriveNodeRequest {
            drive_space_id,
            drive_node_id,
        })
        .await
        .map_err(|error| ApiError::internal("knowledge_drive_node_tree_failed", error.to_string()))?
        .ok_or_else(|| {
            ApiError::not_found(
                "drive_node_not_found",
                "drive node was not found for import resolution".to_string(),
            )
        })?;
    let locator = node.object_locator.ok_or_else(|| {
        ApiError::invalid_request(
            "drive_object_locator_missing",
            "active drive object locator is not available for the requested node".to_string(),
        )
    })?;

    if is_blank(Some(request.drive_storage_provider_id.as_str())) {
        request.drive_storage_provider_id = locator.storage_provider_id;
    }
    if is_blank(Some(request.drive_bucket.as_str())) {
        request.drive_bucket = locator.bucket;
    }
    if is_blank(Some(request.drive_object_key.as_str())) {
        request.drive_object_key = locator.object_key;
    }
    Ok(request)
}

pub(crate) fn map_okf_concept_store_error(
    error: sdkwork_intelligence_knowledgebase_service::ports::knowledge_okf_concept_store::KnowledgeOkfConceptStoreError,
) -> ApiError {
    let detail = error.to_string();
    if detail.contains("missing okf concept") {
        ApiError::not_found("okf_concept_not_found", detail)
    } else {
        ApiError::internal("knowledge_okf_concept_store_failed", detail)
    }
}

fn map_okf_bundle_file_error(
    error: sdkwork_intelligence_knowledgebase_service::ports::knowledge_okf_bundle_file_store::KnowledgeOkfBundleFileStoreError,
) -> ApiError {
    ApiError::internal("knowledge_okf_bundle_file_store_failed", error.to_string())
}

fn document_visibility_label(visibility: KnowledgeDocumentVisibility) -> &'static str {
    match visibility {
        KnowledgeDocumentVisibility::Private => "private",
        KnowledgeDocumentVisibility::Space => "space",
        KnowledgeDocumentVisibility::Organization => "organization",
        KnowledgeDocumentVisibility::Public => "public",
    }
}
