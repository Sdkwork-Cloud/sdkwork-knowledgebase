use async_trait::async_trait;
use sdkwork_drive_storage_local::LocalDriveObjectStore;
use sdkwork_intelligence_knowledgebase_repository_sqlx::{
    connect_sqlite_and_install_schema, sqlite_health_check, SqliteIngestionJobStore,
    SqliteKnowledgeAgentProfileStore, SqliteKnowledgeBrowserProjectionStore,
    SqliteKnowledgeChunkRetrievalStore, SqliteKnowledgeDocumentStore,
    SqliteKnowledgeDocumentVersionStore, SqliteKnowledgeDriveObjectRefStore,
    SqliteKnowledgeEmbeddingStore, SqliteKnowledgeIndexStore, SqliteKnowledgeRetrievalProfileStore,
    SqliteKnowledgeSourceStore, SqliteKnowledgeSpaceStore, SqliteKnowledgeWikiFileEntryStore,
    SqliteKnowledgeWikiPageStore,
};
use sdkwork_intelligence_knowledgebase_service::{
    agent::KnowledgeAgentService, agent_chat::KnowledgeAgentChatService,
    embedding_retrieval_backend::ClawRouterEmbeddingRetrievalBackend,
    retrieval::KnowledgeRetrievalService,
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
    connect_sqlite_drive_pool, sqlite_drive_health_check, KnowledgebaseDriveNodeTreeAdapter,
    KnowledgebaseDriveSpaceProvisionerAdapter, KnowledgebaseDriveStorageAdapter,
    KnowledgebaseDriveWorkspaceAdapter, KnowledgebaseKnowledgeAccessControlAdapter,
};
use sqlx::{AnyPool, SqlitePool};
use std::path::PathBuf;
use std::sync::Arc;

use sdkwork_knowledgebase_agent_provider::{
    resolve_claw_router_client_from_env, ClawRouterEmbeddingClient,
};

use crate::{
    agent_chat_runtime::{
        RuntimeKnowledgebaseRetrievalClient, RuntimeLlmWikiKnowledgeClient,
        RuntimeRetrievalPlanResolver, RuntimeSpaceModeResolver,
    },
    build_router_with_shared_app_api_and_readiness,
    hosted::{
        SqliteHostedBrowserService, SqliteHostedDocumentService, SqliteHostedDriveImportService,
        SqliteHostedIngestService, SqliteHostedSpaceService, SqliteHostedWikiService,
    },
    hosted_backend::SqliteHostedBackendApi,
    hosted_open::SqliteHostedOpenApi,
    ApiError, ApiResult, KnowledgeAgentAppService, KnowledgeAppRequestContext,
    KnowledgeRetrievalAppService, ReadinessCheck,
};

const DEFAULT_DRIVE_PROVIDER_ID: &str = "sdkwork-knowledgebase-local";
const DEFAULT_DRIVE_BUCKET: &str = "knowledgebase";

#[derive(Clone)]
pub struct KnowledgebaseSqliteRuntime {
    pool: SqlitePool,
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
    wiki_file_entry_store: Arc<SqliteKnowledgeWikiFileEntryStore>,
    wiki_page_store: Arc<SqliteKnowledgeWikiPageStore>,
    document_store: Arc<SqliteKnowledgeDocumentStore>,
    source_store: Arc<SqliteKnowledgeSourceStore>,
    version_store: Arc<SqliteKnowledgeDocumentVersionStore>,
    object_ref_store: Arc<SqliteKnowledgeDriveObjectRefStore>,
    ingestion_job_store: Arc<SqliteIngestionJobStore>,
    browser_projection_store: Arc<SqliteKnowledgeBrowserProjectionStore>,
    drive_storage: Arc<KnowledgebaseDriveStorageAdapter>,
    drive_space_provisioner: Arc<KnowledgebaseDriveSpaceProvisionerAdapter>,
    drive_tree: Arc<KnowledgebaseDriveNodeTreeAdapter>,
    drive_workspace: Arc<KnowledgebaseDriveWorkspaceAdapter>,
    access_control: Arc<KnowledgebaseKnowledgeAccessControlAdapter>,
}

impl KnowledgebaseSqliteRuntime {
    pub async fn connect(database_url: &str, tenant_id: u64) -> Result<Self, sqlx::Error> {
        let pool = connect_sqlite_and_install_schema(database_url).await?;
        let drive_pool = connect_sqlite_drive_pool(database_url).await?;
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
        pool: SqlitePool,
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
            wiki_file_entry_store: Arc::new(SqliteKnowledgeWikiFileEntryStore::new(
                pool.clone(),
                tenant_id,
            )),
            wiki_page_store: Arc::new(SqliteKnowledgeWikiPageStore::new(pool.clone(), tenant_id)),
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

    pub fn pool(&self) -> &SqlitePool {
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
        sqlite_health_check(&self.pool).await?;
        sqlite_drive_health_check(&self.drive_pool).await
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
                Arc::new(SqliteHostedSpaceService::new(self.clone())),
                Arc::new(SqliteHostedDriveImportService::new(self.clone())),
                Arc::new(SqliteHostedIngestService::new(self.clone())),
                Arc::new(SqliteHostedDocumentService::new(self.clone())),
                Arc::new(SqliteHostedWikiService::new(self.clone())),
                Arc::new(SqliteHostedBrowserService::new(self.clone())),
                Arc::new(SqliteHostedRetrievalService::new(self.clone())),
                Arc::new(SqliteHostedAgentService::new(self.clone())),
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
            SqliteHostedBackendApi::new(self.clone()),
        ))
    }

    pub fn build_open_api_router(&self) -> axum::Router {
        sdkwork_router_knowledgebase_open_api::build_router_with_shared_open_api(Arc::new(
            SqliteHostedOpenApi::new(self.clone()),
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

    pub(crate) fn wiki_page_store(&self) -> &SqliteKnowledgeWikiPageStore {
        &self.wiki_page_store
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

    pub(crate) fn wiki_file_entry_store(&self) -> &SqliteKnowledgeWikiFileEntryStore {
        &self.wiki_file_entry_store
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
pub(crate) struct SqliteHostedRetrievalService {
    runtime: KnowledgebaseSqliteRuntime,
}

impl SqliteHostedRetrievalService {
    pub(crate) fn new(runtime: KnowledgebaseSqliteRuntime) -> Self {
        Self { runtime }
    }

    fn service(&self) -> KnowledgeRetrievalService<'_> {
        self.runtime.retrieval_service()
    }
}

#[async_trait]
impl KnowledgeRetrievalAppService for SqliteHostedRetrievalService {
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
struct SqliteHostedAgentService {
    runtime: KnowledgebaseSqliteRuntime,
}

impl SqliteHostedAgentService {
    fn new(runtime: KnowledgebaseSqliteRuntime) -> Self {
        Self { runtime }
    }
}

#[async_trait]
impl KnowledgeAgentAppService for SqliteHostedAgentService {
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
        let wiki_client = RuntimeLlmWikiKnowledgeClient::new(
            self.runtime.wiki_page_store.clone(),
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
            wiki_client,
            claw_router_client,
            Some(&plan_resolver),
            Some(&space_mode_resolver),
        );
        service.chat(profile_id, request).await.map_err(Into::into)
    }
}

#[derive(Clone)]
struct AgentAndRetrievalHostedApi {
    retrieval: SqliteHostedRetrievalService,
    agent: SqliteHostedAgentService,
}

impl AgentAndRetrievalHostedApi {
    fn new(runtime: KnowledgebaseSqliteRuntime) -> Self {
        Self {
            retrieval: SqliteHostedRetrievalService::new(runtime.clone()),
            agent: SqliteHostedAgentService::new(runtime),
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
