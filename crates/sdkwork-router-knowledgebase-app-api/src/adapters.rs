use async_trait::async_trait;
use sdkwork_knowledgebase_contract::{
    KnowledgeAgentBinding, KnowledgeAgentBindingList, KnowledgeAgentBindingRequest,
    KnowledgeAgentProfile, KnowledgeAgentProfileRequest, KnowledgeBrowserPage,
    KnowledgeContextPack, KnowledgeContextPackRequest, KnowledgeRetrievalRequest,
    KnowledgeRetrievalResult, ListKnowledgeBrowserRequest,
};
use std::sync::Arc;

use crate::{
    ApiResult, KnowledgeAgentAppService, KnowledgeAppApi, KnowledgeAppRequestContext,
    KnowledgeBrowserApi, KnowledgeRetrievalAppService,
};

pub struct BrowserOnlyAppApi {
    browser: Arc<dyn KnowledgeBrowserApi>,
}

impl BrowserOnlyAppApi {
    pub fn new(browser: Arc<dyn KnowledgeBrowserApi>) -> Self {
        Self { browser }
    }
}

#[async_trait]
impl KnowledgeAppApi for BrowserOnlyAppApi {
    async fn list_browser(
        &self,
        request: ListKnowledgeBrowserRequest,
    ) -> ApiResult<KnowledgeBrowserPage> {
        self.browser.list_browser(request).await
    }
}

pub struct RetrievalOnlyAppApi {
    retrieval: Arc<dyn KnowledgeRetrievalAppService>,
}

impl RetrievalOnlyAppApi {
    pub fn new(retrieval: Arc<dyn KnowledgeRetrievalAppService>) -> Self {
        Self { retrieval }
    }
}

#[async_trait]
impl KnowledgeAppApi for RetrievalOnlyAppApi {
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
}

pub struct AgentOnlyAppApi {
    agent: Arc<dyn KnowledgeAgentAppService>,
}

impl AgentOnlyAppApi {
    pub fn new(agent: Arc<dyn KnowledgeAgentAppService>) -> Self {
        Self { agent }
    }
}

#[async_trait]
impl KnowledgeAppApi for AgentOnlyAppApi {
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
}

pub struct AgentAndRetrievalAppApi {
    agent: Arc<dyn KnowledgeAgentAppService>,
    retrieval: Arc<dyn KnowledgeRetrievalAppService>,
}

impl AgentAndRetrievalAppApi {
    pub fn new(
        agent: Arc<dyn KnowledgeAgentAppService>,
        retrieval: Arc<dyn KnowledgeRetrievalAppService>,
    ) -> Self {
        Self { agent, retrieval }
    }
}

#[async_trait]
impl KnowledgeAppApi for AgentAndRetrievalAppApi {
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
}
