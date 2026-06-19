use async_trait::async_trait;
use sdkwork_intelligence_knowledgebase_service::ports::knowledge_drive_storage::{
    KnowledgeDriveStorage, PutKnowledgeObjectRequest,
};
use sdkwork_intelligence_knowledgebase_service::{
    browser::{KnowledgeBrowserAccessContext, KnowledgeBrowserService},
    imports::KnowledgeDriveImportService,
    ingest::KnowledgeApiPayloadIngestService,
    ports::{
        knowledge_document_store::{
            CreateKnowledgeDocumentRecord, KnowledgeDocumentIdentityScope, KnowledgeDocumentStore,
        },
        knowledge_document_version_store::{
            CreateKnowledgeDocumentVersionRecord, KnowledgeDocumentVersionStore,
        },
        knowledge_ingestion_job_store::IngestionJobStore,
        knowledge_source_store::KnowledgeSourceStore,
        knowledge_space_store::KnowledgeSpaceStore,
        knowledge_wiki_file_entry_store::{
            CreateKnowledgeWikiFileEntryRecord, KnowledgeWikiFileEntryStore,
        },
    },
    retrieval::KnowledgeRetrievalService,
    space::KnowledgeSpaceService,
    wiki::{
        KnowledgeWikiFileRegistryService, KnowledgeWikiInitializerService, KnowledgeWikiPageService,
    },
};
use sdkwork_knowledgebase_contract::wiki_file::WikiFileEntryType;
use sdkwork_knowledgebase_contract::{
    CreateKnowledgeDocumentRequest, CreateKnowledgeDocumentVersionRequest,
    CreateKnowledgeSpaceRequest, IngestionJob, KnowledgeBrowserPage, KnowledgeContextPackRequest,
    KnowledgeDocument, KnowledgeDocumentList, KnowledgeDocumentVersion,
    KnowledgeDocumentVersionList, KnowledgeDriveImportRequest, KnowledgeDriveImportResult,
    KnowledgeIngestRequest, KnowledgeRetrievalRequest, KnowledgeSpace, KnowledgeWikiFileEntry,
    KnowledgeWikiPageRevisionList, ListKnowledgeBrowserRequest, PublishKnowledgeWikiPageRequest,
    WikiContextPackRequest, WikiFileAnswerRequest, WikiIndexDocument, WikiLogDocument,
    WikiPageSummary, WikiPageSummaryList, WikiPageType, WikiQueryRequest, WikiQueryResult,
    WikiSchemaDocument,
};

use crate::{
    hosted_support::{
        default_retrieval_methods, format_retrieval_answer, page_to_summary,
        read_managed_wiki_text, space_binding, wiki_answer_slug, wiki_not_initialized_detail,
        wiki_paths,
    },
    runtime::KnowledgebaseRuntime,
    ApiError, ApiResult, KnowledgeAppRequestContext, KnowledgeBrowserApi,
    KnowledgeDocumentAppService, KnowledgeDriveImportAppService, KnowledgeIngestAppService,
    KnowledgeSpaceAppService, KnowledgeWikiAppService,
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
        request: CreateKnowledgeSpaceRequest,
    ) -> ApiResult<KnowledgeSpace> {
        let file_registry =
            KnowledgeWikiFileRegistryService::new(self.runtime.wiki_file_entry_store());
        let wiki_initializer = KnowledgeWikiInitializerService::new(self.runtime.drive_storage())
            .with_registry(&file_registry);
        let service = KnowledgeSpaceService::new(self.runtime.space_store(), &wiki_initializer)
            .with_drive_context(self.runtime.tenant_id_str(), self.runtime.operator_id())
            .with_drive_space_provisioner(self.runtime.drive_space_provisioner())
            .with_access_control(self.runtime.access_control());
        service.create_space(request).await.map_err(Into::into)
    }

    async fn retrieve_space(&self, space_id: u64) -> ApiResult<KnowledgeSpace> {
        self.runtime
            .space_store()
            .get_space(space_id)
            .await
            .map_err(Into::into)
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
    async fn create_ingest(&self, request: KnowledgeIngestRequest) -> ApiResult<IngestionJob> {
        use sdkwork_intelligence_knowledgebase_service::ingest::{
            KnowledgeApiMarkdownIndexService, KnowledgeIngestionService,
        };
        use sdkwork_knowledgebase_contract::ingest::IngestionJobState;

        let space_id = request.space_id;
        let title = request.title.clone();
        let payload_markdown = request.payload_markdown.clone();

        let service = KnowledgeApiPayloadIngestService::new(
            self.runtime.drive_storage(),
            self.runtime.ingestion_job_store(),
        );
        let result = service
            .ingest_markdown_payload(request)
            .await
            .map_err(ApiError::from)?;
        let mut job = result.job;
        if job.state == IngestionJobState::Queued {
            let ingestion = KnowledgeIngestionService::new(self.runtime.ingestion_job_store());
            job = ingestion
                .mark_running(job.id)
                .await
                .map_err(ApiError::from)?;

            let source = self
                .runtime
                .source_store()
                .create_or_get_source(
                    sdkwork_intelligence_knowledgebase_service::ports::knowledge_source_store::CreateKnowledgeSourceRecord {
                        space_id,
                        source_type: sdkwork_knowledgebase_contract::source::KnowledgeSourceType::Api,
                        provider: Some("api-ingest".to_string()),
                        drive_bucket: None,
                        drive_prefix: Some(format!("inbox/api/{}", job.id)),
                    },
                )
                .await
                .map_err(ApiError::from)?;

            let indexer = KnowledgeApiMarkdownIndexService::new(
                self.runtime.document_store(),
                self.runtime.version_store(),
                self.runtime.object_ref_store(),
                self.runtime.chunk_store(),
            );
            let index_result = match indexer
                .index_payload_markdown(
                    space_id,
                    source.id,
                    &title,
                    &payload_markdown,
                    &result.payload_object_ref,
                )
                .await
            {
                Ok(index_result) => index_result,
                Err(error) => {
                    let _ = ingestion.mark_failed(job.id, format!("{error:?}")).await;
                    return Err(ApiError::from(error));
                }
            };

            let _ = self
                .runtime
                .try_embed_document_version(space_id, index_result.document_version_id)
                .await;

            job = ingestion
                .mark_succeeded(job.id)
                .await
                .map_err(ApiError::from)?;
            self.runtime.try_append_ingest_succeeded_outbox(&job).await;
        }
        Ok(job)
    }

    async fn retrieve_ingest(&self, ingest_id: u64) -> ApiResult<IngestionJob> {
        self.runtime
            .ingestion_job_store()
            .get_job(ingest_id)
            .await
            .map_err(Into::into)
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
        request: KnowledgeDriveImportRequest,
    ) -> ApiResult<KnowledgeDriveImportResult> {
        let service = KnowledgeDriveImportService::new(
            self.runtime.drive_storage(),
            self.runtime.source_store(),
            self.runtime.document_store(),
            self.runtime.object_ref_store(),
            self.runtime.version_store(),
            self.runtime.ingestion_job_store(),
        );
        let result = service
            .import_drive_object(request)
            .await
            .map_err(ApiError::from)?;

        let pipeline = sdkwork_intelligence_knowledgebase_service::ingest::KnowledgeIngestionJobWorkerService::new(
            self.runtime.ingestion_job_store(),
            self.runtime.drive_storage(),
            self.runtime.chunk_store(),
        );
        if let Ok(pipeline_result) = pipeline.process_drive_import_result(&result).await {
            if let Some(index_result) = pipeline_result.index_result {
                let _ = self
                    .runtime
                    .try_embed_document_version(
                        result.document.space_id,
                        index_result.document_version_id,
                    )
                    .await;
            }
            if pipeline_result.job.state
                == sdkwork_knowledgebase_contract::ingest::IngestionJobState::Succeeded
            {
                self.runtime
                    .try_append_ingest_succeeded_outbox(&pipeline_result.job)
                    .await;
            }
            return Ok(KnowledgeDriveImportResult {
                job: pipeline_result.job,
                ..result
            });
        }

        Ok(result)
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
    async fn list_documents(&self) -> ApiResult<KnowledgeDocumentList> {
        let items = self
            .runtime
            .document_store()
            .list_active_documents(200)
            .await
            .map_err(ApiError::from)?;
        Ok(KnowledgeDocumentList { items })
    }

    async fn create_document(
        &self,
        request: CreateKnowledgeDocumentRequest,
    ) -> ApiResult<KnowledgeDocument> {
        if request.title.trim().is_empty() {
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

    async fn retrieve_document(&self, document_id: u64) -> ApiResult<KnowledgeDocument> {
        self.runtime
            .document_store()
            .get_document_by_id(document_id)
            .await
            .map_err(Into::into)
    }

    async fn update_document(
        &self,
        document_id: u64,
        request: CreateKnowledgeDocumentRequest,
    ) -> ApiResult<KnowledgeDocument> {
        if request.title.trim().is_empty() {
            return Err(ApiError::invalid_request(
                "invalid_knowledge_document_request",
                "title is required",
            ));
        }
        self.runtime
            .document_store()
            .update_document_metadata(
                document_id,
                request.title,
                request.mime_type,
                request.language,
            )
            .await
            .map_err(Into::into)
    }

    async fn delete_document(&self, document_id: u64) -> ApiResult<()> {
        self.runtime
            .document_store()
            .soft_delete_document(document_id)
            .await
            .map_err(Into::into)
    }

    async fn list_document_versions(
        &self,
        document_id: u64,
    ) -> ApiResult<KnowledgeDocumentVersionList> {
        let items = self
            .runtime
            .version_store()
            .list_versions_for_document(document_id)
            .await
            .map_err(map_document_version_error)?;
        Ok(KnowledgeDocumentVersionList { items })
    }

    async fn create_document_version(
        &self,
        document_id: u64,
        request: CreateKnowledgeDocumentVersionRequest,
    ) -> ApiResult<KnowledgeDocumentVersion> {
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
    ) -> ApiResult<KnowledgeBrowserPage> {
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
        service
            .list(
                Some(KnowledgeBrowserAccessContext {
                    tenant_id: context.tenant_id,
                    actor_id,
                }),
                request,
            )
            .await
            .map_err(Into::into)
    }
}

#[derive(Clone)]
pub(crate) struct HostedWikiService {
    runtime: KnowledgebaseRuntime,
}

impl HostedWikiService {
    pub fn new(runtime: KnowledgebaseRuntime) -> Self {
        Self { runtime }
    }

    fn wiki_page_service(&self) -> KnowledgeWikiPageService<'_> {
        KnowledgeWikiPageService::new(
            self.runtime.drive_storage(),
            self.runtime.object_ref_store(),
            self.runtime.wiki_page_store(),
        )
        .with_file_entry_store(self.runtime.wiki_file_entry_store())
        .with_drive_workspace(self.runtime.drive_workspace())
    }

    fn retrieval_service(&self) -> KnowledgeRetrievalService<'_> {
        KnowledgeRetrievalService::new(
            self.runtime.retrieval_store(),
            self.runtime.retrieval_store(),
        )
    }

    async fn resolve_wiki_space(&self) -> ApiResult<KnowledgeSpace> {
        self.runtime
            .space_store()
            .find_first_wiki_initialized_space()
            .await
            .map_err(ApiError::from)?
            .ok_or_else(|| {
                ApiError::not_found("wiki_space_not_initialized", wiki_not_initialized_detail())
            })
    }
}

#[async_trait]
impl KnowledgeWikiAppService for HostedWikiService {
    async fn list_wiki_pages(&self) -> ApiResult<WikiPageSummaryList> {
        let items = self
            .runtime
            .wiki_page_store()
            .list_all_page_summaries()
            .await
            .map_err(map_wiki_page_store_error)?;
        Ok(WikiPageSummaryList { items })
    }

    async fn retrieve_wiki_page(&self, page_id: u64) -> ApiResult<WikiPageSummary> {
        let page = self
            .runtime
            .wiki_page_store()
            .get_page_by_id(page_id)
            .await
            .map_err(map_wiki_page_store_error)?;
        Ok(page_to_summary(page))
    }

    async fn list_wiki_page_revisions(
        &self,
        page_id: u64,
    ) -> ApiResult<KnowledgeWikiPageRevisionList> {
        let items = self
            .runtime
            .wiki_page_store()
            .list_page_revisions(page_id)
            .await
            .map_err(map_wiki_page_store_error)?;
        Ok(KnowledgeWikiPageRevisionList { items })
    }

    async fn retrieve_wiki_index(&self) -> ApiResult<WikiIndexDocument> {
        let paths = wiki_paths();
        let markdown =
            read_managed_wiki_text(self.runtime.drive_storage(), paths.index_md, "wiki_index")
                .await?;
        Ok(WikiIndexDocument { markdown })
    }

    async fn retrieve_wiki_log(&self) -> ApiResult<WikiLogDocument> {
        let paths = wiki_paths();
        let markdown =
            read_managed_wiki_text(self.runtime.drive_storage(), paths.log_md, "wiki_log").await?;
        Ok(WikiLogDocument { markdown })
    }

    async fn retrieve_wiki_schema(&self) -> ApiResult<WikiSchemaDocument> {
        let paths = wiki_paths();
        let agents_markdown =
            read_managed_wiki_text(self.runtime.drive_storage(), paths.agents_md, "wiki_schema")
                .await?;
        let schema_yaml = read_managed_wiki_text(
            self.runtime.drive_storage(),
            paths.schema_yaml,
            "wiki_schema",
        )
        .await?;
        Ok(WikiSchemaDocument {
            agents_markdown,
            schema_yaml,
        })
    }

    async fn create_wiki_query(&self, request: WikiQueryRequest) -> ApiResult<WikiQueryResult> {
        if request.space_id == 0 {
            return Err(ApiError::invalid_request(
                "invalid_wiki_query_request",
                "space_id is required",
            ));
        }
        if request.query.trim().is_empty() {
            return Err(ApiError::invalid_request(
                "invalid_wiki_query_request",
                "query is required",
            ));
        }

        let retrieval = self
            .retrieval_service()
            .retrieve(KnowledgeRetrievalRequest {
                tenant_id: self.runtime.tenant_id(),
                actor_id: None,
                query: request.query,
                retrieval_profile_id: None,
                bindings: vec![space_binding(request.space_id)],
                methods: default_retrieval_methods(),
                top_k: Some(8),
                include_citations: true,
                include_trace: true,
                context_budget_tokens: None,
                metadata: vec![],
            })
            .await
            .map_err(ApiError::from)?;

        Ok(WikiQueryResult {
            answer_markdown: format_retrieval_answer(&retrieval.hits),
            trace_id: Some(retrieval.retrieval_id.to_string()),
        })
    }

    async fn file_wiki_query_answer(
        &self,
        query_id: u64,
        request: WikiFileAnswerRequest,
    ) -> ApiResult<WikiQueryResult> {
        if request.title.trim().is_empty() {
            return Err(ApiError::invalid_request(
                "invalid_wiki_file_answer_request",
                "title is required",
            ));
        }
        if request.answer_markdown.trim().is_empty() {
            return Err(ApiError::invalid_request(
                "invalid_wiki_file_answer_request",
                "answer_markdown is required",
            ));
        }

        let space = self.resolve_wiki_space().await?;
        let summary = request.title.clone();
        let publication = self
            .wiki_page_service()
            .publish_page(
                PublishKnowledgeWikiPageRequest {
                    space_id: space.id,
                    slug: wiki_answer_slug(query_id),
                    title: request.title,
                    page_type: WikiPageType::Answer,
                    summary,
                    markdown: request.answer_markdown.clone(),
                    source_count: 1,
                    tags: vec!["wiki-query".to_string()],
                    actor: self.runtime.operator_id().to_string(),
                },
                space.drive_space_id.as_deref(),
            )
            .await
            .map_err(ApiError::from)?;

        Ok(WikiQueryResult {
            answer_markdown: request.answer_markdown,
            trace_id: Some(publication.page.id.to_string()),
        })
    }

    async fn create_wiki_context_pack(
        &self,
        request: WikiContextPackRequest,
    ) -> ApiResult<KnowledgeWikiFileEntry> {
        if request.space_id == 0 {
            return Err(ApiError::invalid_request(
                "invalid_wiki_context_pack_request",
                "space_id is required",
            ));
        }

        let query = request.query.unwrap_or_default();
        let context_pack = self
            .retrieval_service()
            .create_context_pack(KnowledgeContextPackRequest {
                tenant_id: self.runtime.tenant_id(),
                actor_id: None,
                query: query.clone(),
                retrieval_profile_id: None,
                bindings: vec![space_binding(request.space_id)],
                context_budget_tokens: 4096,
                include_citations: true,
                memory_policy_ref: None,
            })
            .await
            .map_err(ApiError::from)?;

        let body = serde_json::to_vec_pretty(&context_pack).map_err(|error| {
            ApiError::internal("wiki_context_pack_serialization_failed", error.to_string())
        })?;
        let logical_path = format!("context_packs/cp-{}.json", context_pack.context_pack_id);
        let object_ref = self
            .runtime
            .drive_storage()
            .put_object(PutKnowledgeObjectRequest {
                logical_path: logical_path.clone(),
                object_role: "context_pack".to_string(),
                content_type: "application/json; charset=utf-8".to_string(),
                body,
                checksum_sha256_hex: None,
            })
            .await?;

        self.runtime
            .wiki_file_entry_store()
            .create_file_entry(CreateKnowledgeWikiFileEntryRecord {
                space_id: request.space_id,
                logical_path,
                entry_type: WikiFileEntryType::ContextPack,
                artifact_role: object_ref.object_role.clone(),
                drive_bucket: object_ref.bucket.clone(),
                drive_object_key: object_ref.object_key.clone(),
                checksum_sha256_hex: object_ref.checksum_sha256_hex.clone(),
            })
            .await
            .map_err(map_wiki_file_entry_error)
    }
}

fn map_document_version_error(
    error: sdkwork_intelligence_knowledgebase_service::ports::knowledge_document_version_store::KnowledgeDocumentVersionStoreError,
) -> ApiError {
    ApiError::internal("knowledge_document_version_store_failed", error.to_string())
}

fn map_wiki_page_store_error(
    error: sdkwork_intelligence_knowledgebase_service::ports::knowledge_wiki_page_store::KnowledgeWikiPageStoreError,
) -> ApiError {
    let detail = error.to_string();
    if detail.contains("missing wiki page") {
        ApiError::not_found("wiki_page_not_found", detail)
    } else {
        ApiError::internal("knowledge_wiki_page_store_failed", detail)
    }
}

fn map_wiki_file_entry_error(
    error: sdkwork_intelligence_knowledgebase_service::ports::knowledge_wiki_file_entry_store::KnowledgeWikiFileEntryStoreError,
) -> ApiError {
    ApiError::internal("knowledge_wiki_file_entry_store_failed", error.to_string())
}
