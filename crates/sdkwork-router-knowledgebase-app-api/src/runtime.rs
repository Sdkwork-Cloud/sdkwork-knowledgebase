use async_trait::async_trait;
use sdkwork_drive_storage_local::LocalDriveObjectStore;
use sdkwork_intelligence_knowledgebase_repository_sqlx::{
    connect_knowledgebase_and_install_schema, knowledgebase_health_check,
    SqliteContextBindingStore, SqliteIngestionJobStore, SqliteKnowledgeAgentProfileStore,
    SqliteKnowledgeBrowserProjectionStore, SqliteKnowledgeChunkRetrievalStore,
    SqliteKnowledgeChunkStore, SqliteKnowledgeDocumentStore, SqliteKnowledgeDocumentVersionStore,
    SqliteKnowledgeDriveObjectRefStore, SqliteKnowledgeEmbeddingStore, SqliteKnowledgeIndexStore,
    SqliteKnowledgeOkfBundleFileStore, SqliteKnowledgeOkfCandidateStore,
    SqliteKnowledgeOkfConceptLinkStore,
    SqliteKnowledgeOkfConceptStore, SqliteKnowledgeOutboxStore,
    SqliteKnowledgeRetrievalProfileStore, SqliteKnowledgeSourceStore, SqliteKnowledgeSpaceStore,
};
use sdkwork_intelligence_knowledgebase_service::{
    agent::KnowledgeAgentService, agent_chat::KnowledgeAgentChatService,
    embedding_retrieval_backend::ClawRouterEmbeddingRetrievalBackend,
    retrieval::KnowledgeRetrievalService,
};
use sdkwork_knowledgebase_contract::agent_chat::{
    KnowledgeAgentChatRequest, KnowledgeAgentChatResponse,
};
use sdkwork_knowledgebase_contract::ingest::IngestionJob;
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
use sqlx::AnyPool;
use std::path::PathBuf;
use std::sync::Arc;

use sdkwork_knowledgebase_agent_provider::{
    resolve_claw_router_client_from_env, ClawRouterEmbeddingClient,
};

use crate::{
    agent_chat_runtime::{
        RuntimeKnowledgebaseRetrievalClient, RuntimeOkfKnowledgeClient,
        RuntimeRetrievalPlanResolver, RuntimeSpaceModeResolver,
    },
    build_router_with_shared_app_api_and_readiness,
    hosted::{
        HostedBrowserService, HostedDocumentService, HostedDriveImportService, HostedIngestService,
        HostedOkfService, HostedSpaceService,
    },
    hosted_backend::HostedBackendApi,
    hosted_context_binding::HostedContextBindingService,
    hosted_open::HostedOpenApi,
    hosted_upload::HostedUploadSessionService,
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
    tenant_id_str: String,
    operator_id: String,
    retrieval_store: Arc<SqliteKnowledgeChunkRetrievalStore>,
    embedding_retrieval_backend:
        Option<Arc<ClawRouterEmbeddingRetrievalBackend<SqliteKnowledgeChunkRetrievalStore>>>,
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
    outbox_store: Arc<SqliteKnowledgeOutboxStore>,
    chunk_store: Arc<SqliteKnowledgeChunkStore>,
    context_binding_store: Arc<SqliteContextBindingStore>,
    browser_projection_store: Arc<SqliteKnowledgeBrowserProjectionStore>,
    drive_storage: Arc<KnowledgebaseDriveStorageAdapter>,
    drive_space_provisioner: Arc<KnowledgebaseDriveSpaceProvisionerAdapter>,
    drive_tree: Arc<KnowledgebaseDriveNodeTreeAdapter>,
    drive_workspace: Arc<KnowledgebaseDriveWorkspaceAdapter>,
    access_control: Arc<KnowledgebaseKnowledgeAccessControlAdapter>,
}

impl KnowledgebaseRuntime {
    pub async fn connect(database_url: &str, tenant_id: u64) -> Result<Self, sqlx::Error> {
        let pool = connect_knowledgebase_and_install_schema(database_url).await?;
        let drive_pool = connect_knowledgebase_drive_pool(database_url).await?;
        Ok(Self::from_pools(
            pool,
            drive_pool,
            tenant_id,
            default_organization_id(),
            default_operator_id(),
            default_drive_storage_root(),
        ))
    }

    fn from_pools(
        pool: AnyPool,
        drive_pool: AnyPool,
        tenant_id: u64,
        organization_id: u64,
        operator_id: String,
        drive_storage_root: PathBuf,
    ) -> Self {
        let tenant_id_str = tenant_id.to_string();
        let object_store = Arc::new(LocalDriveObjectStore::new(drive_storage_root));
        let drive_storage = Arc::new(KnowledgebaseDriveStorageAdapter::new(
            object_store,
            DEFAULT_DRIVE_PROVIDER_ID,
            DEFAULT_DRIVE_BUCKET,
            format!("knowledge/{tenant_id_str}"),
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

        let retrieval_store = Arc::new(SqliteKnowledgeChunkRetrievalStore::new(
            pool.clone(),
            tenant_id,
        ));
        let embedding_retrieval_backend =
            resolve_claw_router_client_from_env().ok().map(|client| {
                Arc::new(ClawRouterEmbeddingRetrievalBackend::new(
                    retrieval_store.as_ref().clone(),
                    ClawRouterEmbeddingClient::new(Arc::new(client)),
                ))
            });

        Self {
            retrieval_store,
            embedding_retrieval_backend,
            retrieval_profile_store: Arc::new(SqliteKnowledgeRetrievalProfileStore::new(
                pool.clone(),
                tenant_id,
            )),
            index_store: Arc::new(SqliteKnowledgeIndexStore::new(pool.clone(), tenant_id)),
            embedding_store: Arc::new(SqliteKnowledgeEmbeddingStore::new(pool.clone(), tenant_id)),
            agent_store: Arc::new(SqliteKnowledgeAgentProfileStore::new(
                pool.clone(),
                tenant_id,
            )),
            space_store: Arc::new(SqliteKnowledgeSpaceStore::new(
                pool.clone(),
                tenant_id,
                organization_id,
            )),
            okf_bundle_file_store: Arc::new(SqliteKnowledgeOkfBundleFileStore::new(
                pool.clone(),
                tenant_id,
            )),
            okf_concept_store: Arc::new(SqliteKnowledgeOkfConceptStore::new(
                pool.clone(),
                tenant_id,
            )),
            okf_concept_link_store: Arc::new(SqliteKnowledgeOkfConceptLinkStore::new(
                pool.clone(),
                tenant_id,
            )),
            okf_candidate_store: Arc::new(SqliteKnowledgeOkfCandidateStore::new(
                pool.clone(),
                tenant_id,
            )),
            document_store: Arc::new(SqliteKnowledgeDocumentStore::new(pool.clone(), tenant_id)),
            source_store: Arc::new(SqliteKnowledgeSourceStore::new(pool.clone(), tenant_id)),
            version_store: Arc::new(SqliteKnowledgeDocumentVersionStore::new(
                pool.clone(),
                tenant_id,
            )),
            object_ref_store: Arc::new(SqliteKnowledgeDriveObjectRefStore::new(
                pool.clone(),
                tenant_id,
            )),
            ingestion_job_store: Arc::new(SqliteIngestionJobStore::new(pool.clone(), tenant_id)),
            outbox_store: Arc::new(SqliteKnowledgeOutboxStore::new(pool.clone(), tenant_id)),
            chunk_store: Arc::new(SqliteKnowledgeChunkStore::new(pool.clone(), tenant_id)),
            context_binding_store: Arc::new(SqliteContextBindingStore::new(pool.clone())),
            browser_projection_store: Arc::new(SqliteKnowledgeBrowserProjectionStore::new(
                pool.clone(),
                tenant_id,
            )),
            pool,
            drive_pool,
            tenant_id,
            tenant_id_str,
            operator_id,
            drive_storage,
            drive_space_provisioner,
            drive_tree,
            drive_workspace,
            access_control,
        }
    }

    pub fn pool(&self) -> &AnyPool {
        &self.pool
    }

    pub fn tenant_id(&self) -> u64 {
        self.tenant_id
    }

    pub(crate) fn retrieval_service(&self) -> KnowledgeRetrievalService<'_> {
        match &self.embedding_retrieval_backend {
            Some(backend) => {
                KnowledgeRetrievalService::new(backend.as_ref(), self.retrieval_store.as_ref())
            }
            None => KnowledgeRetrievalService::new(
                self.retrieval_store.as_ref(),
                self.retrieval_store.as_ref(),
            ),
        }
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
                Arc::new(HostedIngestService::new(self.clone())),
                Arc::new(HostedDocumentService::new(self.clone())),
                Arc::new(HostedOkfService::new(self.clone())),
                Arc::new(HostedBrowserService::new(self.clone())),
                Arc::new(HostedRetrievalService::new(self.clone())),
                Arc::new(HostedAgentService::new(self.clone())),
                Arc::new(HostedContextBindingService::new(self.clone())),
                Arc::new(HostedUploadSessionService::new(self.clone())),
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
        sdkwork_router_knowledgebase_backend_api::build_router_with_shared_backend_api(Arc::new(
            HostedBackendApi::new(self.clone()),
        ))
    }

    pub fn build_open_api_router(&self) -> axum::Router {
        sdkwork_router_knowledgebase_open_api::build_router_with_shared_open_api(Arc::new(
            HostedOpenApi::new(self.clone()),
        ))
    }

    pub async fn build_backend_router_with_web_framework(&self) -> axum::Router {
        sdkwork_router_knowledgebase_backend_api::wrap_router_with_web_framework_from_env(
            self.build_backend_router(),
        )
        .await
    }

    pub async fn build_open_api_router_with_web_framework(&self) -> axum::Router {
        sdkwork_router_knowledgebase_open_api::wrap_router_with_web_framework_from_env(
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

    pub(crate) fn outbox_store(&self) -> &SqliteKnowledgeOutboxStore {
        &self.outbox_store
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

    pub(crate) async fn try_append_ingest_succeeded_outbox(&self, job: &IngestionJob) {
        use sdkwork_intelligence_knowledgebase_service::ports::knowledge_outbox_store::{
            AppendOutboxEventRecord, KnowledgeOutboxStore,
        };

        let payload_json = serde_json::json!({
            "spaceId": job.space_id,
            "sourceType": job.source_type,
            "idempotencyKey": job.idempotency_key,
            "state": format!("{:?}", job.state).to_ascii_lowercase(),
        })
        .to_string();
        let _ = self
            .outbox_store()
            .append_event(AppendOutboxEventRecord {
                aggregate_type: "ingestion_job".to_string(),
                aggregate_id: job.id,
                event_type: "knowledge.ingest.succeeded".to_string(),
                payload_json,
            })
            .await;
    }

    pub async fn publish_pending_outbox_events(&self, limit: u32) -> usize {
        use sdkwork_intelligence_knowledgebase_service::outbox::KnowledgeOutboxPublisherService;

        KnowledgeOutboxPublisherService::new(self.outbox_store())
            .publish_pending(limit)
            .await
            .map(|result| result.published)
            .unwrap_or(0)
    }

    pub async fn process_queued_ingestion_jobs(&self, limit: u32) -> usize {
        use sdkwork_intelligence_knowledgebase_service::ingest::KnowledgeIngestionJobWorkerService;

        KnowledgeIngestionJobWorkerService::new(
            self.ingestion_job_store(),
            self.drive_storage(),
            self.chunk_store(),
        )
        .process_queued_jobs(limit)
        .await
        .map(|result| result.processed)
        .unwrap_or(0)
    }
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
}

#[async_trait]
impl KnowledgeRetrievalAppService for HostedRetrievalService {
    async fn retrieve(
        &self,
        request: KnowledgeRetrievalRequest,
    ) -> ApiResult<KnowledgeRetrievalResult> {
        self.service().retrieve(request).await.map_err(Into::into)
    }

    async fn retrieve_retrieval(
        &self,
        context: KnowledgeAppRequestContext,
        retrieval_id: u64,
    ) -> ApiResult<KnowledgeRetrievalResult> {
        if context.tenant_id != self.runtime.tenant_id {
            return Err(ApiError::new(
                axum::http::StatusCode::FORBIDDEN,
                "tenant_id_mismatch",
                "authenticated tenant does not match configured runtime tenant",
            ));
        }
        self.service()
            .retrieve_persisted(context.tenant_id, retrieval_id)
            .await
            .map_err(Into::into)
    }

    async fn create_context_pack(
        &self,
        request: KnowledgeContextPackRequest,
    ) -> ApiResult<KnowledgeContextPack> {
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
        request: KnowledgeAgentProfileRequest,
    ) -> ApiResult<KnowledgeAgentProfile> {
        let retrieval = self.runtime.retrieval_service();
        let service = KnowledgeAgentService::new(self.runtime.agent_store.as_ref(), &retrieval);
        service.create_profile(request).await.map_err(Into::into)
    }

    async fn retrieve_profile(&self, profile_id: u64) -> ApiResult<KnowledgeAgentProfile> {
        let retrieval = self.runtime.retrieval_service();
        let service = KnowledgeAgentService::new(self.runtime.agent_store.as_ref(), &retrieval);
        service
            .retrieve_profile(profile_id)
            .await
            .map_err(Into::into)
    }

    async fn update_profile(
        &self,
        profile_id: u64,
        request: KnowledgeAgentProfileRequest,
    ) -> ApiResult<KnowledgeAgentProfile> {
        let retrieval = self.runtime.retrieval_service();
        let service = KnowledgeAgentService::new(self.runtime.agent_store.as_ref(), &retrieval);
        service
            .update_profile(profile_id, request)
            .await
            .map_err(Into::into)
    }

    async fn delete_profile(&self, profile_id: u64) -> ApiResult<()> {
        let retrieval = self.runtime.retrieval_service();
        let service = KnowledgeAgentService::new(self.runtime.agent_store.as_ref(), &retrieval);
        service.delete_profile(profile_id).await.map_err(Into::into)
    }

    async fn list_bindings(&self, profile_id: u64) -> ApiResult<KnowledgeAgentBindingList> {
        let retrieval = self.runtime.retrieval_service();
        let service = KnowledgeAgentService::new(self.runtime.agent_store.as_ref(), &retrieval);
        service.list_bindings(profile_id).await.map_err(Into::into)
    }

    async fn create_binding(
        &self,
        profile_id: u64,
        request: KnowledgeAgentBindingRequest,
    ) -> ApiResult<KnowledgeAgentBinding> {
        let retrieval = self.runtime.retrieval_service();
        let service = KnowledgeAgentService::new(self.runtime.agent_store.as_ref(), &retrieval);
        service
            .create_binding(profile_id, request)
            .await
            .map_err(Into::into)
    }

    async fn update_binding(
        &self,
        profile_id: u64,
        binding_id: u64,
        request: KnowledgeAgentBindingRequest,
    ) -> ApiResult<KnowledgeAgentBinding> {
        let retrieval = self.runtime.retrieval_service();
        let service = KnowledgeAgentService::new(self.runtime.agent_store.as_ref(), &retrieval);
        service
            .update_binding(profile_id, binding_id, request)
            .await
            .map_err(Into::into)
    }

    async fn delete_binding(&self, profile_id: u64, binding_id: u64) -> ApiResult<()> {
        let retrieval = self.runtime.retrieval_service();
        let service = KnowledgeAgentService::new(self.runtime.agent_store.as_ref(), &retrieval);
        service
            .delete_binding(profile_id, binding_id)
            .await
            .map_err(Into::into)
    }

    async fn preview_retrieval(
        &self,
        profile_id: u64,
        request: KnowledgeRetrievalRequest,
    ) -> ApiResult<KnowledgeRetrievalResult> {
        let retrieval = self.runtime.retrieval_service();
        let service = KnowledgeAgentService::new(self.runtime.agent_store.as_ref(), &retrieval);
        service
            .preview_retrieval(profile_id, request)
            .await
            .map_err(Into::into)
    }

    async fn create_agent_chat(
        &self,
        profile_id: u64,
        request: KnowledgeAgentChatRequest,
    ) -> ApiResult<KnowledgeAgentChatResponse> {
        let retrieval_client = RuntimeKnowledgebaseRetrievalClient::new(self.runtime.clone());
        let okf_client = RuntimeOkfKnowledgeClient::new(
            self.runtime.okf_concept_store.clone(),
            self.runtime.drive_storage.clone(),
        );
        let claw_router_client = resolve_claw_router_client_from_env()
            .ok()
            .map(std::sync::Arc::new);
        let retrieval = self.runtime.retrieval_service();
        let plan_resolver =
            RuntimeRetrievalPlanResolver::new(self.runtime.retrieval_profile_store.clone());
        let space_mode_resolver = RuntimeSpaceModeResolver::new(self.runtime.space_store.clone());
        let service = KnowledgeAgentChatService::new(
            self.runtime.agent_store.as_ref(),
            &retrieval,
            retrieval_client,
            okf_client,
            claw_router_client,
            Some(&plan_resolver),
            Some(&space_mode_resolver),
        );
        service.chat(profile_id, request).await.map_err(Into::into)
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
        request: KnowledgeRetrievalRequest,
    ) -> ApiResult<KnowledgeRetrievalResult> {
        self.retrieval.retrieve(request).await
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
        request: KnowledgeContextPackRequest,
    ) -> ApiResult<KnowledgeContextPack> {
        self.retrieval.create_context_pack(request).await
    }

    async fn create_agent_profile(
        &self,
        request: KnowledgeAgentProfileRequest,
    ) -> ApiResult<KnowledgeAgentProfile> {
        self.agent.create_profile(request).await
    }

    async fn retrieve_agent_profile(&self, profile_id: u64) -> ApiResult<KnowledgeAgentProfile> {
        self.agent.retrieve_profile(profile_id).await
    }

    async fn update_agent_profile(
        &self,
        profile_id: u64,
        request: KnowledgeAgentProfileRequest,
    ) -> ApiResult<KnowledgeAgentProfile> {
        self.agent.update_profile(profile_id, request).await
    }

    async fn delete_agent_profile(&self, profile_id: u64) -> ApiResult<()> {
        self.agent.delete_profile(profile_id).await
    }

    async fn list_agent_profile_bindings(
        &self,
        profile_id: u64,
    ) -> ApiResult<KnowledgeAgentBindingList> {
        self.agent.list_bindings(profile_id).await
    }

    async fn create_agent_profile_binding(
        &self,
        profile_id: u64,
        request: KnowledgeAgentBindingRequest,
    ) -> ApiResult<KnowledgeAgentBinding> {
        self.agent.create_binding(profile_id, request).await
    }

    async fn update_agent_profile_binding(
        &self,
        profile_id: u64,
        binding_id: u64,
        request: KnowledgeAgentBindingRequest,
    ) -> ApiResult<KnowledgeAgentBinding> {
        self.agent
            .update_binding(profile_id, binding_id, request)
            .await
    }

    async fn delete_agent_profile_binding(
        &self,
        profile_id: u64,
        binding_id: u64,
    ) -> ApiResult<()> {
        self.agent.delete_binding(profile_id, binding_id).await
    }

    async fn create_agent_profile_retrieval_preview(
        &self,
        profile_id: u64,
        request: KnowledgeRetrievalRequest,
    ) -> ApiResult<KnowledgeRetrievalResult> {
        self.agent.preview_retrieval(profile_id, request).await
    }

    async fn create_agent_chat(
        &self,
        profile_id: u64,
        request: KnowledgeAgentChatRequest,
    ) -> ApiResult<KnowledgeAgentChatResponse> {
        self.agent.create_agent_chat(profile_id, request).await
    }
}
