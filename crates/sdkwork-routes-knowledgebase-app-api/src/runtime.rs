use async_trait::async_trait;
use sdkwork_drive_storage_local::LocalDriveObjectStore;
use sdkwork_intelligence_knowledgebase_repository_sqlx::{
    connect_knowledgebase_and_install_schema, connect_postgres_pool,
    default_knowledge_id_generator, is_postgres_database_url,
    keyword_search_backend_for_database_url, knowledgebase_health_check, KnowledgeAuditEventRecord,
    KnowledgeAuditEventStore, PgVectorKnowledgeRetrievalBackend, PgVectorLayeredRetrievalBackend,
    SqliteCommerceStore, SqliteContextBindingStore, SqliteDriveImportMetadataStore,
    SqliteIngestionJobStore, SqliteKnowledgeAgentProfileStore, SqliteKnowledgeAuditEventStore,
    SqliteKnowledgeBrowserProjectionStore, SqliteKnowledgeChunkRetrievalStore,
    SqliteKnowledgeChunkStore, SqliteKnowledgeDocumentStore, SqliteKnowledgeDocumentVersionStore,
    SqliteKnowledgeDriveObjectRefStore, SqliteKnowledgeEmbeddingStore, SqliteKnowledgeIndexStore,
    SqliteKnowledgeOkfBundleFileStore, SqliteKnowledgeOkfCandidateStore,
    SqliteKnowledgeOkfConceptLinkStore, SqliteKnowledgeOkfConceptStore, SqliteKnowledgeOutboxStore,
    SqliteKnowledgeRetrievalProfileStore, SqliteKnowledgeSourceStore, SqliteKnowledgeSpaceStore,
    SqliteMarkdownIndexMetadataStore, SqliteOkfConceptRevisionMetadataStore,
};
use sdkwork_intelligence_knowledgebase_service::{
    agent::KnowledgeAgentService,
    agent_chat::KnowledgeAgentChatService,
    embedding_retrieval_backend::SharedKnowledgeRetrievalBackend,
    knowledge_engine::{
        build_default_registry, DefaultKnowledgeEngineRegistry, KnowledgeEngineRuntimeDeps,
        KnowledgeEngineSpaceResolver,
    },
    ports::{
        knowledge_chunk_store::KnowledgeChunkStore,
        knowledge_drive_object_ref_store::KnowledgeDriveObjectRefStore,
        knowledge_drive_storage::KnowledgeDriveStorage,
        knowledge_outbox_store::KnowledgeOutboxStore,
        knowledge_retrieval_trace_store::KnowledgeRetrievalTraceStore,
        knowledge_space_store::KnowledgeSpaceStore,
    },
    retrieval::{KnowledgeRetrievalService, KnowledgeRetrievalServiceError},
};
use sdkwork_knowledgebase_contract::agent_chat::{
    KnowledgeAgentChatRequest, KnowledgeAgentChatResponse,
};
use sdkwork_knowledgebase_contract::rag::{
    KnowledgeAgentBinding, KnowledgeAgentBindingList, KnowledgeAgentBindingRequest,
    KnowledgeAgentProfile, KnowledgeAgentProfileRequest, KnowledgeContextPack,
    KnowledgeContextPackRequest, KnowledgeRetrievalRequest, KnowledgeRetrievalResult,
};
use sdkwork_knowledgebase_drive::{
    connect_knowledgebase_drive_pool, knowledgebase_drive_health_check,
    KnowledgebaseDriveNodeTreeAdapter, KnowledgebaseDriveSpaceProvisionerAdapter,
    KnowledgebaseDriveStorageAdapter, KnowledgebaseDriveWorkspaceAdapter,
    KnowledgebaseKnowledgeAccessControlAdapter,
};
use sdkwork_utils_rust::is_blank;
use sqlx::AnyPool;
use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::Arc;

use sdkwork_knowledgebase_agent_provider::{
    resolve_claw_router_client_from_env, ClawRouterEmbeddingClient,
};

use crate::{
    agent_chat_runtime::{
        RuntimeKnowledgebaseRetrievalClient, RuntimeOkfKnowledgeClient,
        RuntimeRetrievalPlanResolver, RuntimeSpaceKnowledgeEngineClient, RuntimeSpaceModeResolver,
    },
    build_router_with_shared_app_api_and_readiness,
    hosted::{
        HostedBrowserService, HostedDocumentService, HostedDriveImportService,
        HostedGitImportService, HostedIngestService, HostedOkfService, HostedSpaceService,
    },
    hosted_access::{
        ensure_runtime_tenant, require_actor_id, require_agent_binding_space_access,
        require_agent_profile_space_access, require_bindings_space_access, require_space_access,
    },
    hosted_backend::HostedBackendApi,
    hosted_commerce::HostedCommerceService,
    hosted_context_binding::HostedContextBindingService,
    hosted_open::HostedOpenApi,
    hosted_upload::HostedUploadSessionService,
    hosted_wechat::HostedWechatService,
    ApiError, ApiResult, KnowledgeAgentAppService, KnowledgeAppRequestContext,
    KnowledgeRetrievalAppService, ReadinessCheck,
};

const DEFAULT_DRIVE_PROVIDER_ID: &str = "sdkwork-knowledgebase-local";
const DEFAULT_DRIVE_BUCKET: &str = "knowledgebase";

#[derive(Clone)]
pub struct KnowledgebaseRuntime {
    pool: AnyPool,
    drive_pool: AnyPool,
    tenant_id: u64,
    organization_id: u64,
    tenant_id_str: String,
    operator_id: String,
    retrieval_store: Arc<SqliteKnowledgeChunkRetrievalStore>,
    retrieval_backend: SharedKnowledgeRetrievalBackend,
    retrieval_profile_store: Arc<SqliteKnowledgeRetrievalProfileStore>,
    index_store: Arc<SqliteKnowledgeIndexStore>,
    embedding_store: Arc<SqliteKnowledgeEmbeddingStore>,
    agent_store: Arc<SqliteKnowledgeAgentProfileStore>,
    space_store: Arc<SqliteKnowledgeSpaceStore>,
    okf_bundle_file_store: Arc<SqliteKnowledgeOkfBundleFileStore>,
    okf_concept_store: Arc<SqliteKnowledgeOkfConceptStore>,
    okf_concept_link_store: Arc<SqliteKnowledgeOkfConceptLinkStore>,
    okf_candidate_store: Arc<SqliteKnowledgeOkfCandidateStore>,
    document_store: Arc<SqliteKnowledgeDocumentStore>,
    source_store: Arc<SqliteKnowledgeSourceStore>,
    version_store: Arc<SqliteKnowledgeDocumentVersionStore>,
    object_ref_store: Arc<SqliteKnowledgeDriveObjectRefStore>,
    ingestion_job_store: Arc<SqliteIngestionJobStore>,
    drive_import_metadata_store: Arc<SqliteDriveImportMetadataStore>,
    markdown_index_metadata_store: Arc<SqliteMarkdownIndexMetadataStore>,
    outbox_store: Arc<SqliteKnowledgeOutboxStore>,
    outbox_dispatcher: Arc<dyn sdkwork_intelligence_knowledgebase_service::ports::knowledge_outbox_dispatcher::KnowledgeOutboxDispatcher>,
    chunk_store: Arc<SqliteKnowledgeChunkStore>,
    context_binding_store: Arc<SqliteContextBindingStore>,
    browser_projection_store: Arc<SqliteKnowledgeBrowserProjectionStore>,
    drive_storage: Arc<KnowledgebaseDriveStorageAdapter>,
    drive_space_provisioner: Arc<KnowledgebaseDriveSpaceProvisionerAdapter>,
    drive_tree: Arc<KnowledgebaseDriveNodeTreeAdapter>,
    drive_workspace: Arc<KnowledgebaseDriveWorkspaceAdapter>,
    access_control: Arc<KnowledgebaseKnowledgeAccessControlAdapter>,
    knowledge_engines: Arc<DefaultKnowledgeEngineRegistry>,
    commerce_store: Arc<SqliteCommerceStore>,
}

impl KnowledgebaseRuntime {
    pub async fn connect(database_url: &str, tenant_id: u64) -> Result<Self, sqlx::Error> {
        let pool = connect_knowledgebase_and_install_schema(database_url).await?;
        let drive_pool = connect_knowledgebase_drive_pool(database_url).await?;
        let pg_pool: Option<sqlx::PgPool> = if is_postgres_database_url(database_url) {
            connect_postgres_pool(database_url).await.ok()
        } else {
            None
        };
        let keyword_backend = keyword_search_backend_for_database_url(database_url);
        let database_engine = if is_postgres_database_url(database_url) {
            sdkwork_database_config::DatabaseEngine::Postgres
        } else {
            sdkwork_database_config::DatabaseEngine::Sqlite
        };
        Ok(Self::from_pools(
            pool,
            drive_pool,
            tenant_id,
            default_organization_id(),
            default_operator_id(),
            default_drive_storage_root(),
            keyword_backend,
            pg_pool,
            database_engine,
        ))
    }

    #[allow(clippy::too_many_arguments)]
    fn from_pools(
        pool: AnyPool,
        drive_pool: AnyPool,
        tenant_id: u64,
        organization_id: u64,
        operator_id: String,
        drive_storage_root: PathBuf,
        keyword_backend: sdkwork_intelligence_knowledgebase_repository_sqlx::KeywordSearchBackend,
        pg_pool: Option<sqlx::PgPool>,
        database_engine: sdkwork_database_config::DatabaseEngine,
    ) -> Self {
        let tenant_id_str = tenant_id.to_string();
        let object_store = Arc::new(LocalDriveObjectStore::new(drive_storage_root));
        let drive_storage = Arc::new(KnowledgebaseDriveStorageAdapter::new(
            object_store,
            DEFAULT_DRIVE_PROVIDER_ID,
            DEFAULT_DRIVE_BUCKET,
            tenant_id_str.clone(),
        ));
        let drive_space_provisioner = Arc::new(KnowledgebaseDriveSpaceProvisionerAdapter::new(
            drive_pool.clone(),
        ));
        let drive_tree = Arc::new(KnowledgebaseDriveNodeTreeAdapter::new(
            drive_pool.clone(),
            tenant_id_str.clone(),
        ));
        let drive_workspace = Arc::new(KnowledgebaseDriveWorkspaceAdapter::new(
            drive_pool.clone(),
            tenant_id_str.clone(),
            operator_id.clone(),
        ));
        let access_control = Arc::new(KnowledgebaseKnowledgeAccessControlAdapter::new(
            drive_pool.clone(),
        ));

        let retrieval_store = Arc::new(SqliteKnowledgeChunkRetrievalStore::with_keyword_backend(
            pool.clone(),
            tenant_id,
            keyword_backend,
            default_knowledge_id_generator(),
        ));

        let base_retrieval: SharedKnowledgeRetrievalBackend = if let Some(pg_pool) = pg_pool {
            Arc::new(PgVectorLayeredRetrievalBackend::new(
                retrieval_store.clone(),
                Arc::new(PgVectorKnowledgeRetrievalBackend::new(pg_pool, tenant_id)),
            ))
        } else {
            retrieval_store.clone()
        };

        let retrieval_backend: SharedKnowledgeRetrievalBackend =
            resolve_claw_router_client_from_env()
                .ok()
                .map(|client| {
                    Arc::new(
                        sdkwork_intelligence_knowledgebase_service::embedding_retrieval_backend::ClawRouterEmbeddingRetrievalBackend::new(
                            base_retrieval.clone(),
                            ClawRouterEmbeddingClient::new(Arc::new(client)),
                        ),
                    ) as SharedKnowledgeRetrievalBackend
                })
                .unwrap_or(base_retrieval);

        let okf_concept_store =
            Arc::new(SqliteKnowledgeOkfConceptStore::new(pool.clone(), tenant_id));
        let space_store = Arc::new(SqliteKnowledgeSpaceStore::new(
            pool.clone(),
            tenant_id,
            organization_id,
        ));
        let okf_bundle_file_store = Arc::new(SqliteKnowledgeOkfBundleFileStore::new(
            pool.clone(),
            tenant_id,
        ));
        let okf_concept_link_store = Arc::new(SqliteKnowledgeOkfConceptLinkStore::new(
            pool.clone(),
            tenant_id,
        ));
        let okf_candidate_store = Arc::new(SqliteKnowledgeOkfCandidateStore::new(
            pool.clone(),
            tenant_id,
        ));
        let okf_revision_metadata_store = Arc::new(SqliteOkfConceptRevisionMetadataStore::new(
            pool.clone(),
            tenant_id,
        ));
        let source_store = Arc::new(SqliteKnowledgeSourceStore::new(pool.clone(), tenant_id));
        let object_ref_store = Arc::new(SqliteKnowledgeDriveObjectRefStore::new(
            pool.clone(),
            tenant_id,
        ));
        let document_store = Arc::new(SqliteKnowledgeDocumentStore::new(pool.clone(), tenant_id));
        let index_store = Arc::new(SqliteKnowledgeIndexStore::new(pool.clone(), tenant_id));
        let embedding_store = Arc::new(SqliteKnowledgeEmbeddingStore::new(
            pool.clone(),
            tenant_id,
            database_engine,
        ));
        let rag_embedder = resolve_claw_router_client_from_env()
            .ok()
            .map(|client| ClawRouterEmbeddingClient::new(Arc::new(client)));

        let knowledge_engines = Arc::new(build_default_registry(KnowledgeEngineRuntimeDeps {
            tenant_id,
            okf: KnowledgeEngineRuntimeDeps::okf_from_stores(
                okf_concept_store.clone(),
                drive_storage.clone(),
                okf_revision_metadata_store.clone(),
                object_ref_store.clone(),
                okf_concept_link_store.clone(),
                okf_candidate_store.clone(),
                okf_bundle_file_store.clone(),
                drive_workspace.clone(),
                source_store.clone(),
                space_store.clone(),
            ),
            rag_documents: document_store.clone(),
            retrieval_backend: retrieval_backend.clone(),
            retrieval_traces: retrieval_store.clone(),
            rag_index_store: Some(index_store.clone()),
            rag_embedding_store: Some(embedding_store.clone()),
            rag_embedder,
            external_engines:
                crate::knowledge_engine_adapters::load_runtime_external_adapter_engines(
                    source_store.clone(),
                ),
        }));

        let audit_store = Arc::new(SqliteKnowledgeAuditEventStore::new(pool.clone(), tenant_id));
        sdkwork_knowledgebase_observability::install_audit_persistence(Arc::new(move |event| {
            audit_store.record(KnowledgeAuditEventRecord {
                event_type: event.event_type,
                actor_type: event.actor_type,
                actor_id: event.actor_id,
                resource_type: event.resource_type,
                resource_id: event.resource_id,
                result: event.result,
                request_id: None,
                trace_id: None,
                payload: event.payload,
            });
        }));

        Self {
            retrieval_store,
            retrieval_backend,
            retrieval_profile_store: Arc::new(SqliteKnowledgeRetrievalProfileStore::new(
                pool.clone(),
                tenant_id,
            )),
            index_store,
            embedding_store,
            agent_store: Arc::new(SqliteKnowledgeAgentProfileStore::new(
                pool.clone(),
                tenant_id,
            )),
            space_store,
            okf_bundle_file_store,
            okf_concept_store,
            okf_concept_link_store,
            okf_candidate_store,
            document_store,
            source_store,
            version_store: Arc::new(SqliteKnowledgeDocumentVersionStore::new(
                pool.clone(),
                tenant_id,
            )),
            object_ref_store,
            ingestion_job_store: Arc::new(SqliteIngestionJobStore::with_keyword_backend(
                pool.clone(),
                tenant_id,
                keyword_backend,
                default_knowledge_id_generator(),
            )),
            drive_import_metadata_store: Arc::new(SqliteDriveImportMetadataStore::new(
                pool.clone(),
                tenant_id,
            )),
            markdown_index_metadata_store: Arc::new(SqliteMarkdownIndexMetadataStore::new(
                pool.clone(),
                tenant_id,
            )),
            outbox_store: Arc::new(
                SqliteKnowledgeOutboxStore::new(pool.clone(), tenant_id).with_postgres_skip_locked_claim(
                    database_engine == sdkwork_database_config::DatabaseEngine::Postgres,
                ),
            ),
            outbox_dispatcher:
                sdkwork_intelligence_knowledgebase_service::outbox::knowledge_outbox_dispatcher_from_env(),
            chunk_store: Arc::new(SqliteKnowledgeChunkStore::with_keyword_backend(
                pool.clone(),
                tenant_id,
                keyword_backend,
                default_knowledge_id_generator(),
            )),
            context_binding_store: Arc::new(SqliteContextBindingStore::new(pool.clone())),
            browser_projection_store: Arc::new(SqliteKnowledgeBrowserProjectionStore::new(
                pool.clone(),
                tenant_id,
            )),
            pool: pool.clone(),
            drive_pool,
            tenant_id,
            organization_id,
            tenant_id_str,
            operator_id,
            drive_storage,
            drive_space_provisioner,
            drive_tree,
            drive_workspace,
            access_control,
            knowledge_engines,
            commerce_store: Arc::new(SqliteCommerceStore::new(pool.clone())),
        }
    }

    pub fn pool(&self) -> &AnyPool {
        &self.pool
    }

    pub fn tenant_id(&self) -> u64 {
        self.tenant_id
    }

    pub fn organization_id(&self) -> u64 {
        self.organization_id
    }

    pub(crate) fn arc_agent_store(&self) -> Arc<SqliteKnowledgeAgentProfileStore> {
        self.agent_store.clone()
    }

    pub(crate) fn arc_retrieval_profile_store(&self) -> Arc<SqliteKnowledgeRetrievalProfileStore> {
        self.retrieval_profile_store.clone()
    }

    pub(crate) fn arc_space_store(&self) -> Arc<SqliteKnowledgeSpaceStore> {
        self.space_store.clone()
    }

    pub(crate) fn commerce_store(&self) -> &SqliteCommerceStore {
        &self.commerce_store
    }

    pub(crate) fn retrieval_service(&self) -> KnowledgeRetrievalService<'_> {
        KnowledgeRetrievalService::new(
            self.retrieval_backend.as_ref(),
            self.retrieval_store.as_ref(),
        )
    }

    pub async fn readiness_check(&self) -> Result<(), sqlx::Error> {
        knowledgebase_health_check(&self.pool).await?;
        knowledgebase_drive_health_check(&self.drive_pool).await?;
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM kb_space_context_binding")
            .fetch_one(&self.pool)
            .await
            .map(|_| ())
    }

    pub fn build_agent_and_retrieval_router(&self) -> axum::Router {
        build_router_with_shared_app_api_and_readiness(
            Arc::new(AgentAndRetrievalHostedApi::new(self.clone())),
            Some(ReadinessCheck::new(self.pool.clone())),
        )
    }

    pub fn build_full_app_router(&self) -> axum::Router {
        use crate::adapters::FullAppApi;

        build_router_with_shared_app_api_and_readiness(
            Arc::new(FullAppApi::new(
                Arc::new(HostedSpaceService::new(self.clone())),
                Arc::new(HostedDriveImportService::new(self.clone())),
                Arc::new(HostedGitImportService::new(self.clone())),
                Arc::new(HostedIngestService::new(self.clone())),
                Arc::new(HostedDocumentService::new(self.clone())),
                Arc::new(HostedOkfService::new(self.clone())),
                Arc::new(HostedBrowserService::new(self.clone())),
                Arc::new(HostedRetrievalService::new(self.clone())),
                Arc::new(HostedAgentService::new(self.clone())),
                Arc::new(HostedContextBindingService::new(self.clone())),
                Arc::new(HostedUploadSessionService::new(self.clone())),
                Arc::new(HostedWechatService::new(self.clone())),
                Arc::new(HostedCommerceService::new(self.clone())),
            )),
            Some(ReadinessCheck::new(self.pool.clone())),
        )
    }

    pub async fn build_full_app_router_with_web_framework(&self) -> axum::Router {
        crate::web_bootstrap::wrap_router_with_web_framework_from_env(self.build_full_app_router())
            .await
    }

    pub async fn build_agent_and_retrieval_router_with_web_framework(&self) -> axum::Router {
        crate::web_bootstrap::wrap_router_with_web_framework_from_env(
            self.build_agent_and_retrieval_router(),
        )
        .await
    }

    pub fn build_backend_router(&self) -> axum::Router {
        sdkwork_routes_knowledgebase_backend_api::build_router_with_shared_backend_api_and_readiness(
            Arc::new(HostedBackendApi::new(self.clone())),
            self.tenant_id(),
            Some(
                sdkwork_routes_knowledgebase_backend_api::DbReadinessCheck::new(self.pool.clone()),
            ),
        )
    }

    pub fn build_open_api_router(&self) -> axum::Router {
        sdkwork_routes_knowledgebase_open_api::build_router_with_shared_open_api_and_readiness(
            Arc::new(HostedOpenApi::new(self.clone())),
            Some(
                sdkwork_routes_knowledgebase_backend_api::DbReadinessCheck::new(self.pool.clone()),
            ),
        )
    }

    pub async fn build_backend_router_with_web_framework(&self) -> axum::Router {
        sdkwork_routes_knowledgebase_backend_api::wrap_router_with_web_framework_from_env(
            self.build_backend_router(),
        )
        .await
    }

    pub async fn build_open_api_router_with_web_framework(&self) -> axum::Router {
        sdkwork_routes_knowledgebase_open_api::wrap_router_with_web_framework_from_env(
            self.build_open_api_router(),
        )
        .await
    }

    pub(crate) fn retrieval_store(&self) -> &SqliteKnowledgeChunkRetrievalStore {
        &self.retrieval_store
    }

    pub(crate) fn retrieval_profile_store(&self) -> &SqliteKnowledgeRetrievalProfileStore {
        &self.retrieval_profile_store
    }

    pub(crate) fn index_store(&self) -> &SqliteKnowledgeIndexStore {
        &self.index_store
    }

    pub(crate) fn embedding_store(&self) -> &SqliteKnowledgeEmbeddingStore {
        &self.embedding_store
    }

    pub(crate) fn okf_concept_store(&self) -> &SqliteKnowledgeOkfConceptStore {
        &self.okf_concept_store
    }

    pub(crate) fn okf_concept_link_store(&self) -> &SqliteKnowledgeOkfConceptLinkStore {
        &self.okf_concept_link_store
    }

    pub(crate) fn okf_candidate_store(&self) -> &SqliteKnowledgeOkfCandidateStore {
        &self.okf_candidate_store
    }

    pub(crate) fn space_store(&self) -> &SqliteKnowledgeSpaceStore {
        &self.space_store
    }

    pub(crate) fn document_store(&self) -> &SqliteKnowledgeDocumentStore {
        &self.document_store
    }

    pub(crate) fn ingestion_job_store(&self) -> &SqliteIngestionJobStore {
        &self.ingestion_job_store
    }

    pub(crate) fn drive_import_metadata_store(&self) -> &SqliteDriveImportMetadataStore {
        &self.drive_import_metadata_store
    }

    pub(crate) fn markdown_index_metadata_store(&self) -> &SqliteMarkdownIndexMetadataStore {
        &self.markdown_index_metadata_store
    }

    pub(crate) fn outbox_store(&self) -> &SqliteKnowledgeOutboxStore {
        &self.outbox_store
    }

    pub(crate) fn outbox_dispatcher(
        &self,
    ) -> &dyn sdkwork_intelligence_knowledgebase_service::ports::knowledge_outbox_dispatcher::KnowledgeOutboxDispatcher
    {
        self.outbox_dispatcher.as_ref()
    }

    pub(crate) fn chunk_store(&self) -> &SqliteKnowledgeChunkStore {
        &self.chunk_store
    }

    pub(crate) fn context_binding_store(&self) -> &SqliteContextBindingStore {
        &self.context_binding_store
    }

    pub(crate) fn okf_bundle_file_store(&self) -> &SqliteKnowledgeOkfBundleFileStore {
        &self.okf_bundle_file_store
    }

    pub(crate) fn source_store(&self) -> &SqliteKnowledgeSourceStore {
        &self.source_store
    }

    pub(crate) fn knowledge_engine_registry(&self) -> &DefaultKnowledgeEngineRegistry {
        &self.knowledge_engines
    }

    pub(crate) fn knowledge_engines(&self) -> &DefaultKnowledgeEngineRegistry {
        &self.knowledge_engines
    }

    pub(crate) fn knowledge_engine_space_resolver(
        &self,
    ) -> KnowledgeEngineSpaceResolver<DefaultKnowledgeEngineRegistry> {
        KnowledgeEngineSpaceResolver::new(
            self.knowledge_engines.clone(),
            self.space_store.clone(),
            self.source_store.clone(),
        )
    }

    pub async fn resolve_knowledge_engine_implementation_id_for_space(
        &self,
        space_id: u64,
    ) -> Result<String, String> {
        self.knowledge_engine_space_resolver()
            .resolve_for_space(space_id, None)
            .await
            .map(|engine| engine.descriptor().implementation_id)
            .map_err(|error| error.to_string())
    }

    pub async fn read_knowledge_engine_document_for_space(
        &self,
        space_id: u64,
        document_id: &str,
    ) -> Result<sdkwork_knowledgebase_contract::knowledge_engine::KnowledgeEngineDocument, String>
    {
        use sdkwork_knowledgebase_contract::knowledge_engine::KnowledgeEngineReadRequest;

        self.knowledge_engine_space_resolver()
            .resolve_for_space(space_id, None)
            .await
            .map_err(|error| error.to_string())?
            .read_document(KnowledgeEngineReadRequest {
                tenant_id: self.tenant_id,
                space_id,
                document_id: document_id.to_string(),
            })
            .await
            .map_err(|error| error.to_string())
    }

    pub(crate) fn okf_bundle_engine_for_space(
        &self,
        space_id: u64,
        implementation_id: &str,
    ) -> Result<
        &sdkwork_intelligence_knowledgebase_service::knowledge_engine::OkfNativeKnowledgeEngine,
        crate::ApiError,
    > {
        use sdkwork_knowledgebase_contract::knowledge_engine::KnowledgeEngineId;

        if implementation_id != KnowledgeEngineId::OKF_NATIVE {
            return Err(crate::ApiError::invalid_request(
                "okf_bundle_engine_required",
                format!(
                    "space {space_id} resolves to {implementation_id}, expected {}",
                    KnowledgeEngineId::OKF_NATIVE
                ),
            ));
        }
        Ok(self.knowledge_engines().okf_native())
    }

    pub async fn resolve_okf_bundle_engine_for_space(
        &self,
        space_id: u64,
    ) -> Result<
        &sdkwork_intelligence_knowledgebase_service::knowledge_engine::OkfNativeKnowledgeEngine,
        crate::ApiError,
    > {
        let implementation_id = self
            .resolve_knowledge_engine_implementation_id_for_space(space_id)
            .await
            .map_err(|detail| crate::ApiError::internal("okf_engine_resolve_failed", detail))?;
        self.okf_bundle_engine_for_space(space_id, &implementation_id)
    }

    pub async fn ensure_bindings_support_rag_retrieval(
        &self,
        bindings: &[sdkwork_knowledgebase_contract::rag::KnowledgeRetrievalBinding],
    ) -> Result<(), crate::ApiError> {
        use sdkwork_knowledgebase_contract::rag::KnowledgeAgentKnowledgeMode;

        for binding in bindings {
            let space = self.space_store().get_space(binding.space_id).await?;
            if space.knowledge_mode != KnowledgeAgentKnowledgeMode::Rag {
                return Err(crate::ApiError::invalid_request(
                    "rag_retrieval_mode_required",
                    format!(
                        "space {} uses {:?} knowledge mode; use okf query/context-pack or external engine APIs instead of RAG retrievals",
                        binding.space_id, space.knowledge_mode
                    ),
                ));
            }
        }
        Ok(())
    }

    pub async fn search_knowledge_engine_for_space(
        &self,
        space_id: u64,
        query: &str,
        top_k: u32,
    ) -> Result<sdkwork_knowledgebase_contract::knowledge_engine::KnowledgeEngineSearchResult, String>
    {
        use sdkwork_knowledgebase_contract::knowledge_engine::KnowledgeEngineSearchRequest;

        self.knowledge_engine_space_resolver()
            .resolve_for_space(space_id, None)
            .await
            .map_err(|error| error.to_string())?
            .search(KnowledgeEngineSearchRequest {
                tenant_id: self.tenant_id,
                space_id,
                query: query.to_string(),
                top_k,
            })
            .await
            .map_err(|error| error.to_string())
    }

    pub(crate) fn object_ref_store(&self) -> &SqliteKnowledgeDriveObjectRefStore {
        &self.object_ref_store
    }

    pub(crate) fn version_store(&self) -> &SqliteKnowledgeDocumentVersionStore {
        &self.version_store
    }

    pub(crate) fn browser_projection_store(&self) -> &SqliteKnowledgeBrowserProjectionStore {
        &self.browser_projection_store
    }

    pub(crate) fn drive_storage(&self) -> &KnowledgebaseDriveStorageAdapter {
        &self.drive_storage
    }

    pub(crate) fn drive_space_provisioner(&self) -> &KnowledgebaseDriveSpaceProvisionerAdapter {
        &self.drive_space_provisioner
    }

    pub(crate) fn drive_tree(&self) -> &KnowledgebaseDriveNodeTreeAdapter {
        &self.drive_tree
    }

    pub(crate) fn drive_workspace(&self) -> &KnowledgebaseDriveWorkspaceAdapter {
        &self.drive_workspace
    }

    pub(crate) fn access_control(&self) -> &KnowledgebaseKnowledgeAccessControlAdapter {
        &self.access_control
    }

    pub(crate) fn tenant_id_str(&self) -> &str {
        &self.tenant_id_str
    }

    pub(crate) fn operator_id(&self) -> &str {
        &self.operator_id
    }

    pub(crate) async fn try_embed_document_version(
        &self,
        space_id: u64,
        document_version_id: u64,
    ) -> Option<usize> {
        use sdkwork_intelligence_knowledgebase_service::ingest::KnowledgePostIngestEmbeddingService;
        use sdkwork_knowledgebase_agent_provider::{
            resolve_claw_router_client_from_env, ClawRouterEmbeddingClient,
        };

        let client = resolve_claw_router_client_from_env().ok()?;
        let index = self
            .index_store()
            .get_or_create_active_vector_index(space_id, 0)
            .await
            .ok()?;
        let embedder = ClawRouterEmbeddingClient::new(Arc::new(client));
        let service = KnowledgePostIngestEmbeddingService::new(
            self.chunk_store(),
            self.embedding_store(),
            embedder,
        );
        service
            .embed_document_version(self.tenant_id, &index, document_version_id)
            .await
            .ok()
    }

    pub async fn publish_pending_outbox_events(&self, limit: u32) -> usize {
        use sdkwork_intelligence_knowledgebase_service::outbox::KnowledgeOutboxPublisherService;

        let _ = self.requeue_failed_outbox_events(limit).await;
        KnowledgeOutboxPublisherService::new(
            self.tenant_id(),
            self.outbox_store(),
            self.outbox_dispatcher(),
        )
        .publish_pending(limit)
        .await
        .map(|result| result.published)
        .unwrap_or(0)
    }

    pub async fn requeue_failed_outbox_events(&self, limit: u32) -> usize {
        let max_retry_count = std::env::var("SDKWORK_KNOWLEDGEBASE_OUTBOX_MAX_RETRIES")
            .ok()
            .and_then(|value| value.parse::<u32>().ok())
            .unwrap_or(5)
            .clamp(1, 32);
        self.outbox_store()
            .requeue_failed_events(limit, max_retry_count)
            .await
            .unwrap_or(0)
    }

    pub async fn read_document_content_markdown(
        &self,
        document_id: u64,
    ) -> Result<sdkwork_knowledgebase_contract::document::KnowledgeDocumentContent, String> {
        use sdkwork_intelligence_knowledgebase_service::ports::knowledge_drive_storage::KnowledgeObjectRef;
        use sdkwork_knowledgebase_contract::document::KnowledgeDocumentContent;

        let versions = self
            .version_store()
            .list_versions_for_document(document_id)
            .await
            .map_err(|error| error.to_string())?;
        let latest = versions
            .last()
            .ok_or_else(|| "document has no versions".to_string())?;
        let object_ref = self
            .object_ref_store()
            .get_object_ref_by_id(latest.original_object_ref_id)
            .await
            .map_err(|error| error.to_string())?;
        let storage_ref = KnowledgeObjectRef {
            storage_provider_id: object_ref.drive_storage_provider_id.clone(),
            bucket: object_ref.drive_bucket.clone(),
            object_key: object_ref.drive_object_key.clone(),
            logical_path: object_ref
                .logical_path
                .clone()
                .unwrap_or_else(|| object_ref.drive_object_key.clone()),
            object_role: object_ref.object_role.clone(),
            content_type: object_ref
                .content_type
                .clone()
                .unwrap_or_else(|| "text/markdown; charset=utf-8".to_string()),
            size_bytes: object_ref.size_bytes,
            checksum_sha256_hex: object_ref.checksum_sha256_hex.clone(),
            etag: object_ref.drive_etag.clone(),
            version_id: object_ref.drive_object_version.clone(),
        };

        if let Ok(content) = self.drive_storage().get_object_text(&storage_ref).await {
            if !is_blank(Some(content.as_str())) {
                let content_source = "drive_object".to_string();
                return Ok(KnowledgeDocumentContent {
                    document_id,
                    content_markdown: content,
                    content_source: content_source.clone(),
                    content_version: build_document_content_version(
                        latest,
                        object_ref.drive_etag.as_deref(),
                        &content_source,
                    ),
                });
            }
        }

        let chunks = self
            .chunk_store()
            .list_chunk_texts_for_document_version(latest.id)
            .await
            .map_err(|error| error.to_string())?;
        if chunks.is_empty() {
            return Err("document content is not available".to_string());
        }

        let content_source = "chunk_concat".to_string();
        Ok(KnowledgeDocumentContent {
            document_id,
            content_markdown: chunks.join("\n\n"),
            content_source: content_source.clone(),
            content_version: build_document_content_version(latest, None, &content_source),
        })
    }

    pub async fn process_queued_ingestion_jobs(&self, limit: u32) -> usize {
        use sdkwork_intelligence_knowledgebase_service::ingest::KnowledgeIngestionJobWorkerService;

        KnowledgeIngestionJobWorkerService::new(self.ingestion_job_store(), self.drive_storage())
            .process_queued_jobs(limit)
            .await
            .map(|result| result.processed)
            .unwrap_or(0)
    }
}

fn build_document_content_version(
    version: &sdkwork_knowledgebase_contract::document::KnowledgeDocumentVersion,
    drive_etag: Option<&str>,
    content_source: &str,
) -> String {
    if let Some(etag) = drive_etag.filter(|value| !value.is_empty()) {
        return format!("{content_source}:etag:{etag}");
    }
    if let Some(checksum) = version
        .checksum_sha256_hex
        .as_deref()
        .filter(|value| !value.is_empty())
    {
        return format!("{content_source}:v{}:{checksum}", version.id);
    }
    format!("{content_source}:v{}:{}", version.id, version.version_no)
}

fn default_organization_id() -> u64 {
    std::env::var("SDKWORK_KNOWLEDGEBASE_ORGANIZATION_ID")
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(0)
}

fn default_operator_id() -> String {
    std::env::var("SDKWORK_KNOWLEDGEBASE_OPERATOR_ID").unwrap_or_else(|_| "system".to_string())
}

fn default_drive_storage_root() -> PathBuf {
    std::env::var("SDKWORK_KNOWLEDGEBASE_DRIVE_STORAGE_ROOT")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("data/drive-objects"))
}

#[derive(Clone)]
pub(crate) struct HostedRetrievalService {
    runtime: KnowledgebaseRuntime,
}

impl HostedRetrievalService {
    pub(crate) fn new(runtime: KnowledgebaseRuntime) -> Self {
        Self { runtime }
    }

    fn service(&self) -> KnowledgeRetrievalService<'_> {
        self.runtime.retrieval_service()
    }

    async fn authorize_retrieval_request(
        &self,
        context: &KnowledgeAppRequestContext,
        request: &KnowledgeRetrievalRequest,
    ) -> ApiResult<()> {
        ensure_runtime_tenant(&self.runtime, context)?;
        require_bindings_space_access(&self.runtime, context, &request.bindings).await?;
        self.runtime
            .ensure_bindings_support_rag_retrieval(&request.bindings)
            .await
    }

    async fn authorize_retrieval_trace(
        &self,
        context: &KnowledgeAppRequestContext,
        retrieval_id: u64,
    ) -> ApiResult<()> {
        ensure_runtime_tenant(&self.runtime, context)?;
        let _actor_id = require_actor_id(context)?;

        let trace = self
            .runtime
            .retrieval_store()
            .retrieve_trace(context.tenant_id, retrieval_id)
            .await
            .map_err(|error| ApiError::from(KnowledgeRetrievalServiceError::TraceStore(error)))?;

        if let Some(trace_actor_id) = trace.actor_id {
            if context.actor_id != Some(trace_actor_id) {
                return Err(ApiError::new(
                    axum::http::StatusCode::FORBIDDEN,
                    "retrieval_trace_access_denied",
                    "authenticated actor does not own this retrieval trace",
                ));
            }
        }

        let hits = self
            .runtime
            .retrieval_store()
            .list_trace_hits(context.tenant_id, retrieval_id)
            .await
            .map_err(|error| ApiError::from(KnowledgeRetrievalServiceError::TraceStore(error)))?;

        let mut seen = HashSet::new();
        for hit in hits {
            if seen.insert(hit.space_id) {
                require_space_access(&self.runtime, context, hit.space_id).await?;
            }
        }

        Ok(())
    }
}

#[async_trait]
impl KnowledgeRetrievalAppService for HostedRetrievalService {
    async fn retrieve(
        &self,
        context: KnowledgeAppRequestContext,
        mut request: KnowledgeRetrievalRequest,
    ) -> ApiResult<KnowledgeRetrievalResult> {
        self.authorize_retrieval_request(&context, &request).await?;
        request = request.with_actor_id(context.actor_id);
        self.service().retrieve(request).await.map_err(Into::into)
    }

    async fn retrieve_retrieval(
        &self,
        context: KnowledgeAppRequestContext,
        retrieval_id: u64,
    ) -> ApiResult<KnowledgeRetrievalResult> {
        self.authorize_retrieval_trace(&context, retrieval_id)
            .await?;
        self.service()
            .retrieve_persisted(context.tenant_id, retrieval_id)
            .await
            .map_err(Into::into)
    }

    async fn create_context_pack(
        &self,
        context: KnowledgeAppRequestContext,
        mut request: KnowledgeContextPackRequest,
    ) -> ApiResult<KnowledgeContextPack> {
        let retrieval_request = KnowledgeRetrievalRequest {
            tenant_id: request.tenant_id,
            actor_id: request.actor_id,
            query: request.query.clone(),
            retrieval_profile_id: request.retrieval_profile_id,
            bindings: request.bindings.clone(),
            methods: vec![],
            top_k: None,
            include_citations: request.include_citations,
            include_trace: true,
            context_budget_tokens: Some(request.context_budget_tokens),
            metadata: vec![],
        };
        self.authorize_retrieval_request(&context, &retrieval_request)
            .await?;
        request = request
            .with_tenant_id(context.tenant_id)
            .with_actor_id(context.actor_id);
        self.service()
            .create_context_pack(request)
            .await
            .map_err(Into::into)
    }
}

#[derive(Clone)]
struct HostedAgentService {
    runtime: KnowledgebaseRuntime,
}

impl HostedAgentService {
    fn new(runtime: KnowledgebaseRuntime) -> Self {
        Self { runtime }
    }
}

#[async_trait]
impl KnowledgeAgentAppService for HostedAgentService {
    async fn create_profile(
        &self,
        context: KnowledgeAppRequestContext,
        request: KnowledgeAgentProfileRequest,
    ) -> ApiResult<KnowledgeAgentProfile> {
        ensure_runtime_tenant(&self.runtime, &context)?;
        let retrieval = self.runtime.retrieval_service();
        let service = KnowledgeAgentService::new(self.runtime.agent_store.as_ref(), &retrieval);
        service.create_profile(request).await.map_err(Into::into)
    }

    async fn retrieve_profile(
        &self,
        context: KnowledgeAppRequestContext,
        profile_id: u64,
    ) -> ApiResult<KnowledgeAgentProfile> {
        let retrieval = self.runtime.retrieval_service();
        let service = KnowledgeAgentService::new(self.runtime.agent_store.as_ref(), &retrieval);
        let profile = service
            .retrieve_profile(profile_id)
            .await
            .map_err(ApiError::from)?;
        require_agent_profile_space_access(&self.runtime, &context, &profile).await?;
        Ok(profile)
    }

    async fn update_profile(
        &self,
        context: KnowledgeAppRequestContext,
        profile_id: u64,
        request: KnowledgeAgentProfileRequest,
    ) -> ApiResult<KnowledgeAgentProfile> {
        let retrieval = self.runtime.retrieval_service();
        let service = KnowledgeAgentService::new(self.runtime.agent_store.as_ref(), &retrieval);
        let existing = service
            .retrieve_profile(profile_id)
            .await
            .map_err(ApiError::from)?;
        require_agent_profile_space_access(&self.runtime, &context, &existing).await?;
        service
            .update_profile(profile_id, request)
            .await
            .map_err(Into::into)
    }

    async fn delete_profile(
        &self,
        context: KnowledgeAppRequestContext,
        profile_id: u64,
    ) -> ApiResult<()> {
        let retrieval = self.runtime.retrieval_service();
        let service = KnowledgeAgentService::new(self.runtime.agent_store.as_ref(), &retrieval);
        let existing = service
            .retrieve_profile(profile_id)
            .await
            .map_err(ApiError::from)?;
        require_agent_profile_space_access(&self.runtime, &context, &existing).await?;
        service.delete_profile(profile_id).await.map_err(Into::into)
    }

    async fn list_bindings(
        &self,
        context: KnowledgeAppRequestContext,
        profile_id: u64,
    ) -> ApiResult<KnowledgeAgentBindingList> {
        let retrieval = self.runtime.retrieval_service();
        let service = KnowledgeAgentService::new(self.runtime.agent_store.as_ref(), &retrieval);
        let profile = service
            .retrieve_profile(profile_id)
            .await
            .map_err(ApiError::from)?;
        require_agent_profile_space_access(&self.runtime, &context, &profile).await?;
        service.list_bindings(profile_id).await.map_err(Into::into)
    }

    async fn create_binding(
        &self,
        context: KnowledgeAppRequestContext,
        profile_id: u64,
        request: KnowledgeAgentBindingRequest,
    ) -> ApiResult<KnowledgeAgentBinding> {
        let retrieval = self.runtime.retrieval_service();
        let service = KnowledgeAgentService::new(self.runtime.agent_store.as_ref(), &retrieval);
        let profile = service
            .retrieve_profile(profile_id)
            .await
            .map_err(ApiError::from)?;
        require_agent_profile_space_access(&self.runtime, &context, &profile).await?;
        require_agent_binding_space_access(&self.runtime, &context, request.space_id).await?;
        service
            .create_binding(profile_id, request)
            .await
            .map_err(Into::into)
    }

    async fn update_binding(
        &self,
        context: KnowledgeAppRequestContext,
        profile_id: u64,
        binding_id: u64,
        request: KnowledgeAgentBindingRequest,
    ) -> ApiResult<KnowledgeAgentBinding> {
        let retrieval = self.runtime.retrieval_service();
        let service = KnowledgeAgentService::new(self.runtime.agent_store.as_ref(), &retrieval);
        let profile = service
            .retrieve_profile(profile_id)
            .await
            .map_err(ApiError::from)?;
        require_agent_profile_space_access(&self.runtime, &context, &profile).await?;
        require_agent_binding_space_access(&self.runtime, &context, request.space_id).await?;
        service
            .update_binding(profile_id, binding_id, request)
            .await
            .map_err(Into::into)
    }

    async fn delete_binding(
        &self,
        context: KnowledgeAppRequestContext,
        profile_id: u64,
        binding_id: u64,
    ) -> ApiResult<()> {
        let retrieval = self.runtime.retrieval_service();
        let service = KnowledgeAgentService::new(self.runtime.agent_store.as_ref(), &retrieval);
        let profile = service
            .retrieve_profile(profile_id)
            .await
            .map_err(ApiError::from)?;
        require_agent_profile_space_access(&self.runtime, &context, &profile).await?;
        service
            .delete_binding(profile_id, binding_id)
            .await
            .map_err(Into::into)
    }

    async fn preview_retrieval(
        &self,
        context: KnowledgeAppRequestContext,
        profile_id: u64,
        request: KnowledgeRetrievalRequest,
    ) -> ApiResult<KnowledgeRetrievalResult> {
        let retrieval = self.runtime.retrieval_service();
        let service = KnowledgeAgentService::new(self.runtime.agent_store.as_ref(), &retrieval);
        let profile = service
            .retrieve_profile(profile_id)
            .await
            .map_err(ApiError::from)?;
        require_agent_profile_space_access(&self.runtime, &context, &profile).await?;
        require_bindings_space_access(&self.runtime, &context, &request.bindings).await?;
        service
            .preview_retrieval(profile_id, request)
            .await
            .map_err(Into::into)
    }

    async fn create_agent_chat(
        &self,
        context: KnowledgeAppRequestContext,
        profile_id: u64,
        request: KnowledgeAgentChatRequest,
    ) -> ApiResult<KnowledgeAgentChatResponse> {
        let retrieval = self.runtime.retrieval_service();
        let service = KnowledgeAgentService::new(self.runtime.agent_store.as_ref(), &retrieval);
        let profile = service
            .retrieve_profile(profile_id)
            .await
            .map_err(ApiError::from)?;
        require_agent_profile_space_access(&self.runtime, &context, &profile).await?;

        let retrieval_client = RuntimeKnowledgebaseRetrievalClient::new(self.runtime.clone());
        let okf_client = RuntimeOkfKnowledgeClient::new(self.runtime.clone());
        let claw_router_client = resolve_claw_router_client_from_env()
            .ok()
            .map(std::sync::Arc::new);
        let retrieval = self.runtime.retrieval_service();
        let plan_resolver =
            RuntimeRetrievalPlanResolver::new(self.runtime.retrieval_profile_store.clone());
        let space_mode_resolver = RuntimeSpaceModeResolver::new(self.runtime.space_store.clone());
        let space_engine_client =
            Arc::new(RuntimeSpaceKnowledgeEngineClient::new(self.runtime.clone()));
        let chat_service = KnowledgeAgentChatService::new(
            self.runtime.agent_store.as_ref(),
            &retrieval,
            retrieval_client,
            okf_client,
            claw_router_client,
            Some(&plan_resolver),
            Some(&space_mode_resolver),
            Some(space_engine_client),
        );
        chat_service
            .chat(profile_id, request)
            .await
            .map_err(Into::into)
    }
}

#[derive(Clone)]
struct AgentAndRetrievalHostedApi {
    retrieval: HostedRetrievalService,
    agent: HostedAgentService,
}

impl AgentAndRetrievalHostedApi {
    fn new(runtime: KnowledgebaseRuntime) -> Self {
        Self {
            retrieval: HostedRetrievalService::new(runtime.clone()),
            agent: HostedAgentService::new(runtime),
        }
    }
}

#[async_trait::async_trait]
impl crate::KnowledgeAppApi for AgentAndRetrievalHostedApi {
    async fn create_retrieval(
        &self,
        context: KnowledgeAppRequestContext,
        request: KnowledgeRetrievalRequest,
    ) -> ApiResult<KnowledgeRetrievalResult> {
        self.retrieval.retrieve(context, request).await
    }

    async fn retrieve_retrieval(
        &self,
        context: KnowledgeAppRequestContext,
        retrieval_id: u64,
    ) -> ApiResult<KnowledgeRetrievalResult> {
        self.retrieval
            .retrieve_retrieval(context, retrieval_id)
            .await
    }

    async fn create_context_pack(
        &self,
        context: KnowledgeAppRequestContext,
        request: KnowledgeContextPackRequest,
    ) -> ApiResult<KnowledgeContextPack> {
        self.retrieval.create_context_pack(context, request).await
    }

    async fn create_agent_profile(
        &self,
        context: KnowledgeAppRequestContext,
        request: KnowledgeAgentProfileRequest,
    ) -> ApiResult<KnowledgeAgentProfile> {
        self.agent.create_profile(context, request).await
    }

    async fn retrieve_agent_profile(
        &self,
        context: KnowledgeAppRequestContext,
        profile_id: u64,
    ) -> ApiResult<KnowledgeAgentProfile> {
        self.agent.retrieve_profile(context, profile_id).await
    }

    async fn update_agent_profile(
        &self,
        context: KnowledgeAppRequestContext,
        profile_id: u64,
        request: KnowledgeAgentProfileRequest,
    ) -> ApiResult<KnowledgeAgentProfile> {
        self.agent
            .update_profile(context, profile_id, request)
            .await
    }

    async fn delete_agent_profile(
        &self,
        context: KnowledgeAppRequestContext,
        profile_id: u64,
    ) -> ApiResult<()> {
        self.agent.delete_profile(context, profile_id).await
    }

    async fn list_agent_profile_bindings(
        &self,
        context: KnowledgeAppRequestContext,
        profile_id: u64,
    ) -> ApiResult<KnowledgeAgentBindingList> {
        self.agent.list_bindings(context, profile_id).await
    }

    async fn create_agent_profile_binding(
        &self,
        context: KnowledgeAppRequestContext,
        profile_id: u64,
        request: KnowledgeAgentBindingRequest,
    ) -> ApiResult<KnowledgeAgentBinding> {
        self.agent
            .create_binding(context, profile_id, request)
            .await
    }

    async fn update_agent_profile_binding(
        &self,
        context: KnowledgeAppRequestContext,
        profile_id: u64,
        binding_id: u64,
        request: KnowledgeAgentBindingRequest,
    ) -> ApiResult<KnowledgeAgentBinding> {
        self.agent
            .update_binding(context, profile_id, binding_id, request)
            .await
    }

    async fn delete_agent_profile_binding(
        &self,
        context: KnowledgeAppRequestContext,
        profile_id: u64,
        binding_id: u64,
    ) -> ApiResult<()> {
        self.agent
            .delete_binding(context, profile_id, binding_id)
            .await
    }

    async fn create_agent_profile_retrieval_preview(
        &self,
        context: KnowledgeAppRequestContext,
        profile_id: u64,
        request: KnowledgeRetrievalRequest,
    ) -> ApiResult<KnowledgeRetrievalResult> {
        self.agent
            .preview_retrieval(context, profile_id, request)
            .await
    }

    async fn create_agent_chat(
        &self,
        context: KnowledgeAppRequestContext,
        profile_id: u64,
        request: KnowledgeAgentChatRequest,
    ) -> ApiResult<KnowledgeAgentChatResponse> {
        self.agent
            .create_agent_chat(context, profile_id, request)
            .await
    }
}
