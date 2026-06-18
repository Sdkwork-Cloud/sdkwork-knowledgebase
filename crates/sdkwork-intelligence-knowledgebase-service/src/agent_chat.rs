use crate::ports::knowledge_agent_profile_store::{
    KnowledgeAgentProfileStore, KnowledgeAgentProfileStoreError,
};
use crate::retrieval::{KnowledgeRetrievalExecutor, KnowledgeRetrievalServiceError};
use sdkwork_agent_kernel::{
    AgentChatRequest, AgentChatService, AgentManifest, KernelError, ModelProvider, ModelRequest,
    ModelResponse, PolicyDecision, PolicyProvider, PolicyRequest, ProviderHealth, ProviderManifest,
    RuntimeBuilder,
};
use sdkwork_agent_plugin_core::SdkworkKernelPlugin;
use sdkwork_agent_plugin_rig::{ids as rig_ids, rig_agent_manifest, RigKernelPlugin};
use sdkwork_knowledgebase_agent_provider::{
    default_top_k, is_rig_model_provider, resolve_chat_knowledge_mode,
    validate_bindings_support_mode, validate_rag_profile_requirements, ClawRouterChatModelProvider,
    KnowledgeAccessGateway, KnowledgeAccessRequest, KnowledgeAccessRetrievalExecutor,
    KnowledgeRetrievalPlanResolver, KnowledgeSpaceModeResolver, KnowledgebaseRetrievalClient,
    LlmWikiKnowledgeClient, LlmWikiKnowledgeProvider, SdkworkKnowledgebaseProvider,
    CLAW_ROUTER_OPEN_HTTP_URL_ENV, LLM_WIKI_KNOWLEDGE_PROVIDER_ID, RIG_MODEL_PROVIDER_ID,
    SDKWORK_KNOWLEDGEBASE_PROVIDER_ID,
};
use sdkwork_knowledgebase_contract::agent_chat::{
    KnowledgeAgentChatRequest, KnowledgeAgentChatResponse, KnowledgeAgentKnowledgeMode,
};
use sdkwork_knowledgebase_contract::rag::KnowledgeRetrievalRequest;
use std::sync::Arc;
use thiserror::Error;
use uuid::Uuid;

pub const DEFAULT_MODEL_PROVIDER_ID: &str = rig_ids::MODEL_PROVIDER_ID;
pub const DEFAULT_MODEL_ID: &str = rig_ids::DEFAULT_MODEL_ID;
pub const CONTRACT_MODEL_PROVIDER_ID: &str = "provider.model.knowledgebase-contract";

fn resolve_model_provider_id(model_provider_id: &str) -> String {
    if model_provider_id == CONTRACT_MODEL_PROVIDER_ID {
        return model_provider_id.to_string();
    }
    if is_rig_model_provider(model_provider_id) {
        return rig_ids::MODEL_PROVIDER_ID.to_string();
    }
    model_provider_id.to_string()
}

pub struct KnowledgeAgentChatService<'a, R, W> {
    profiles: &'a dyn KnowledgeAgentProfileStore,
    retrieval: &'a dyn KnowledgeRetrievalExecutor,
    retrieval_client: R,
    wiki_client: W,
    claw_router_client: Option<Arc<clawrouter_open_sdk::SdkworkAiClient>>,
    retrieval_plan_resolver: Option<&'a dyn KnowledgeRetrievalPlanResolver>,
    space_mode_resolver: Option<&'a dyn KnowledgeSpaceModeResolver>,
}

impl<'a, R, W> KnowledgeAgentChatService<'a, R, W>
where
    R: KnowledgebaseRetrievalClient + Send + Sync + Clone + 'static,
    W: LlmWikiKnowledgeClient + Send + Sync + Clone + 'static,
{
    pub fn new(
        profiles: &'a dyn KnowledgeAgentProfileStore,
        retrieval: &'a dyn KnowledgeRetrievalExecutor,
        retrieval_client: R,
        wiki_client: W,
        claw_router_client: Option<Arc<clawrouter_open_sdk::SdkworkAiClient>>,
        retrieval_plan_resolver: Option<&'a dyn KnowledgeRetrievalPlanResolver>,
        space_mode_resolver: Option<&'a dyn KnowledgeSpaceModeResolver>,
    ) -> Self {
        Self {
            profiles,
            retrieval,
            retrieval_client,
            wiki_client,
            claw_router_client,
            retrieval_plan_resolver,
            space_mode_resolver,
        }
    }

    pub async fn chat(
        &self,
        profile_id: u64,
        request: KnowledgeAgentChatRequest,
    ) -> Result<KnowledgeAgentChatResponse, KnowledgeAgentChatServiceError> {
        if profile_id == 0 {
            return Err(KnowledgeAgentChatServiceError::InvalidRequest(
                "profile_id is required".to_string(),
            ));
        }
        if request.tenant_id == 0 {
            return Err(KnowledgeAgentChatServiceError::InvalidRequest(
                "tenant_id is required".to_string(),
            ));
        }
        if request.message.trim().is_empty() {
            return Err(KnowledgeAgentChatServiceError::InvalidRequest(
                "message is required".to_string(),
            ));
        }

        let profile =
            self.profiles
                .retrieve_profile(profile_id)
                .await
                .map_err(|error| match error {
                    KnowledgeAgentProfileStoreError::NotFound(id) => {
                        KnowledgeAgentChatServiceError::InvalidRequest(format!(
                            "knowledge agent profile was not found: {id}"
                        ))
                    }
                    KnowledgeAgentProfileStoreError::Conflict(detail) => {
                        KnowledgeAgentChatServiceError::InvalidRequest(detail)
                    }
                    KnowledgeAgentProfileStoreError::Internal(detail) => {
                        KnowledgeAgentChatServiceError::Runtime(detail)
                    }
                })?;

        if profile.tenant_id != request.tenant_id {
            return Err(KnowledgeAgentChatServiceError::InvalidRequest(
                "profile tenant_id must match chat request tenant_id".to_string(),
            ));
        }

        let bindings = sdkwork_knowledgebase_agent_provider::enabled_bindings(&profile.bindings);
        if bindings.is_empty() {
            return Err(KnowledgeAgentChatServiceError::InvalidRequest(
                "agent profile requires at least one enabled knowledge binding".to_string(),
            ));
        }

        let mode = resolve_chat_knowledge_mode(request.mode, profile.knowledge_mode);
        validate_rag_profile_requirements(mode, profile.retrieval_profile_id)
            .map_err(KnowledgeAgentChatServiceError::InvalidRequest)?;

        if let Some(resolver) = self.space_mode_resolver {
            validate_bindings_support_mode(resolver, &bindings, mode)
                .await
                .map_err(KnowledgeAgentChatServiceError::InvalidRequest)?;
        }

        let top_k = default_top_k(&bindings);
        let retrieval_plan = if mode == KnowledgeAgentKnowledgeMode::Rag {
            match self.retrieval_plan_resolver {
                Some(resolver) => resolver
                    .resolve_plan(request.tenant_id, profile.retrieval_profile_id)
                    .await
                    .map_err(KnowledgeAgentChatServiceError::KnowledgeProvider)?,
                None => None,
            }
        } else {
            None
        };

        let access = KnowledgeAccessGateway::new(
            self.wiki_client.clone(),
            RetrievalExecutorAdapter {
                retrieval: self.retrieval,
            },
        );
        let access_result = access
            .fetch(KnowledgeAccessRequest {
                tenant_id: request.tenant_id,
                message: &request.message,
                mode,
                bindings: &bindings,
                top_k,
                retrieval_profile_id: profile.retrieval_profile_id,
                retrieval_methods: vec![],
                retrieval_plan,
            })
            .await
            .map_err(KnowledgeAgentChatServiceError::KnowledgeProvider)?;

        let citations = access_result.citations;
        let retrieval_id = access_result.retrieval_id;
        let knowledge_namespace = access_result.namespace;
        let knowledge_methods = access_result.kernel_methods;

        let model_provider_id = request
            .model_provider_id
            .clone()
            .filter(|value| !value.trim().is_empty())
            .unwrap_or_else(|| profile.model_provider_id.clone());
        let resolved_model_provider_id = resolve_model_provider_id(&model_provider_id);
        let model_id = request
            .model_id
            .clone()
            .filter(|value| !value.trim().is_empty())
            .unwrap_or_else(|| profile.model_id.clone());

        let chat_id = format!("chat.{}", Uuid::new_v4());
        let runtime = build_chat_runtime(
            &resolved_model_provider_id,
            mode,
            self.retrieval_client.clone(),
            self.wiki_client.clone(),
            request.tenant_id,
            self.claw_router_client.clone(),
        )?;

        let mut chat_request =
            AgentChatRequest::new(chat_id.clone(), vec![request.message.clone()])
                .with_provider_id(resolved_model_provider_id.clone())
                .with_model_id(model_id.clone())
                .with_knowledge_query(&request.message)
                .with_knowledge_provider_id(knowledge_provider_id(mode))
                .with_knowledge_tenant_id(request.tenant_id.to_string())
                .with_knowledge_namespace(knowledge_namespace)
                .with_knowledge_top_k(default_top_k(&bindings));

        for method in knowledge_methods {
            chat_request = chat_request.with_knowledge_method(method);
        }

        if let Some(session_id) = request.session_id.clone() {
            chat_request = chat_request.for_session(session_id.clone());
        }

        if !profile.system_instruction.trim().is_empty() {
            chat_request = chat_request.with_metadata(
                "sdkwork.knowledge.system_instruction",
                profile.system_instruction.clone(),
            );
        }

        let response = AgentChatService::new()
            .invoke(&runtime, chat_request)
            .map_err(map_kernel_error)?;

        let answer = response
            .model_response
            .messages
            .join("\n")
            .trim()
            .to_string();

        Ok(KnowledgeAgentChatResponse {
            chat_id,
            answer,
            mode,
            model_provider_id: response.provider_id,
            model_id,
            citations,
            retrieval_id,
            session_id: request.session_id,
        })
    }
}

struct RetrievalExecutorAdapter<'a> {
    retrieval: &'a dyn KnowledgeRetrievalExecutor,
}

#[async_trait::async_trait]
impl KnowledgeAccessRetrievalExecutor for RetrievalExecutorAdapter<'_> {
    async fn retrieve(
        &self,
        request: KnowledgeRetrievalRequest,
    ) -> Result<sdkwork_knowledgebase_contract::KnowledgeRetrievalResult, String> {
        self.retrieval
            .retrieve(request)
            .await
            .map_err(|error| error.to_string())
    }
}

fn knowledge_provider_id(mode: KnowledgeAgentKnowledgeMode) -> &'static str {
    match mode {
        KnowledgeAgentKnowledgeMode::LlmWiki => LLM_WIKI_KNOWLEDGE_PROVIDER_ID,
        KnowledgeAgentKnowledgeMode::Rag => SDKWORK_KNOWLEDGEBASE_PROVIDER_ID,
    }
}

fn build_chat_runtime<R, W>(
    model_provider_id: &str,
    mode: KnowledgeAgentKnowledgeMode,
    retrieval_client: R,
    wiki_client: W,
    tenant_id: u64,
    claw_router_client: Option<Arc<clawrouter_open_sdk::SdkworkAiClient>>,
) -> Result<sdkwork_agent_kernel::AgentRuntime, KnowledgeAgentChatServiceError>
where
    R: KnowledgebaseRetrievalClient + Send + Sync + 'static,
    W: LlmWikiKnowledgeClient + Send + Sync + 'static,
{
    let manifest = chat_agent_manifest(model_provider_id);
    let mut builder = RuntimeBuilder::new("runtime.knowledgebase.chat", manifest);

    if model_provider_id == CONTRACT_MODEL_PROVIDER_ID {
        builder = builder
            .register_model_provider(
                CONTRACT_MODEL_PROVIDER_ID,
                "0.1.0",
                ContractKnowledgeChatModelProvider,
            )
            .register_policy_provider("provider.policy.allow", "0.1.0", AllowPolicyProvider);
    } else if is_rig_model_provider(model_provider_id) {
        let client = claw_router_client.ok_or_else(|| {
            KnowledgeAgentChatServiceError::Runtime(format!(
                "Rig LLM backend requires claw-router SdkworkAiClient ({CLAW_ROUTER_OPEN_HTTP_URL_ENV})"
            ))
        })?;
        builder = builder.register_model_provider(
            rig_ids::MODEL_PROVIDER_ID,
            "0.1.0",
            ClawRouterChatModelProvider::for_rig(client),
        );
        builder = RigKernelPlugin::fail_closed().configure_runtime(builder);
    } else {
        return Err(KnowledgeAgentChatServiceError::InvalidRequest(format!(
            "unsupported model provider id: {model_provider_id}; expected {CONTRACT_MODEL_PROVIDER_ID} or {RIG_MODEL_PROVIDER_ID}"
        )));
    }

    builder = builder
        .register_knowledge_provider(
            SDKWORK_KNOWLEDGEBASE_PROVIDER_ID,
            "0.1.0",
            SdkworkKnowledgebaseProvider::new(retrieval_client, tenant_id),
        )
        .register_knowledge_provider(
            LLM_WIKI_KNOWLEDGE_PROVIDER_ID,
            "0.1.0",
            LlmWikiKnowledgeProvider::new(wiki_client),
        );

    let _ = mode;

    builder
        .bootstrap()
        .map(|bootstrapped| bootstrapped.runtime)
        .map_err(|error| {
            KnowledgeAgentChatServiceError::Runtime(format!(
                "agent runtime bootstrap failed: {error}"
            ))
        })
}

fn chat_agent_manifest(model_provider_id: &str) -> AgentManifest {
    if model_provider_id == CONTRACT_MODEL_PROVIDER_ID {
        knowledgebase_chat_agent_manifest()
    } else {
        rig_agent_manifest()
    }
}

fn knowledgebase_chat_agent_manifest() -> AgentManifest {
    AgentManifest::from_json(
        r#"
{
  "schema_version": "0.1.0",
  "manifest_type": "agent",
  "agent_id": "agent.knowledgebase.chat",
  "name": "knowledgebase-chat",
  "display_name": "Knowledgebase Chat",
  "description": "Knowledge-backed chat agent for SDKWork Knowledgebase.",
  "version": "0.1.0",
  "domain": "intelligence",
  "required_capabilities": [
    { "capability_id": "model.chat", "min_version": "0.1.0" },
    { "capability_id": "policy.evaluate", "min_version": "0.1.0" }
  ],
  "optional_capabilities": [
    { "capability_id": "knowledge.search", "min_version": "0.1.0" }
  ],
  "event_families": ["agent.model.*", "agent.knowledge.*"],
  "owner": { "name": "sdkwork-platform" },
  "status": "candidate"
}
"#,
    )
    .expect("knowledgebase chat manifest parses")
}

fn map_kernel_error(error: KernelError) -> KnowledgeAgentChatServiceError {
    KnowledgeAgentChatServiceError::AgentKernel(error.to_string())
}

#[derive(Debug, Clone)]
struct AllowPolicyProvider;

impl PolicyProvider for AllowPolicyProvider {
    fn provider_manifest(&self) -> ProviderManifest {
        ProviderManifest::new(
            "provider.policy.allow",
            "policy",
            "allow-policy",
            "0.1.0",
            vec!["policy.evaluate".to_string()],
        )
    }

    fn evaluate(
        &self,
        request: PolicyRequest,
    ) -> sdkwork_agent_kernel::KernelResult<PolicyDecision> {
        Ok(PolicyDecision::allow(
            format!("decision.{}", request.policy_request_id),
            request.policy_request_id,
            "provider.policy.allow",
        ))
    }

    fn health(&self) -> ProviderHealth {
        ProviderHealth::available()
    }
}

#[derive(Debug, Clone)]
struct ContractKnowledgeChatModelProvider;

impl ModelProvider for ContractKnowledgeChatModelProvider {
    fn provider_manifest(&self) -> ProviderManifest {
        ProviderManifest::new(
            CONTRACT_MODEL_PROVIDER_ID,
            "model",
            "knowledgebase-contract",
            "0.1.0",
            vec!["model.chat".to_string()],
        )
    }

    fn health(&self) -> ProviderHealth {
        ProviderHealth::available()
    }

    fn invoke(&self, request: ModelRequest) -> sdkwork_agent_kernel::KernelResult<ModelResponse> {
        let context_titles = request
            .context_frames
            .iter()
            .filter_map(|frame| frame.metadata_value("sdkwork.knowledge.title"))
            .collect::<Vec<_>>();

        let answer = if context_titles.is_empty() {
            format!(
                "No knowledge context was attached for this question: {}",
                request.messages.join(" ")
            )
        } else {
            format!(
                "Based on {} knowledge source(s) [{}]: {}",
                context_titles.len(),
                context_titles.join(", "),
                request.messages.join(" ")
            )
        };

        Ok(ModelResponse::text(
            request.model_request_id,
            CONTRACT_MODEL_PROVIDER_ID,
            answer,
        ))
    }
}

#[derive(Debug, Error)]
pub enum KnowledgeAgentChatServiceError {
    #[error("invalid knowledge agent chat request: {0}")]
    InvalidRequest(String),
    #[error("knowledge provider error: {0}")]
    KnowledgeProvider(String),
    #[error(transparent)]
    Retrieval(#[from] KnowledgeRetrievalServiceError),
    #[error("agent runtime error: {0}")]
    Runtime(String),
    #[error("agent kernel error: {0}")]
    AgentKernel(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ports::knowledge_agent_profile_store::{
        KnowledgeAgentProfileStore, KnowledgeAgentProfileStoreError,
    };
    use async_trait::async_trait;
    use sdkwork_agent_kernel::{KnowledgeDocument, KnowledgeDocumentFilter, KnowledgeDocumentKind};
    use sdkwork_knowledgebase_contract::rag::{
        KnowledgeAgentBinding, KnowledgeAgentProfile, KnowledgeAgentProfileRequest,
        KnowledgeAgentStatus, KnowledgeContextFragment, KnowledgeRetrievalMethod,
        KnowledgeRetrievalResult, KnowledgeRetrievalTrace,
    };
    use sdkwork_knowledgebase_contract::wiki::WikiPageSummary;
    use sdkwork_knowledgebase_contract::WikiPageType;

    #[tokio::test]
    async fn chat_defaults_to_llm_wiki_mode_with_citations_and_contract_model() {
        let service = KnowledgeAgentChatService::new(
            &FakeProfileStore,
            &FakeRetrieval,
            FakeRetrievalClient,
            FakeWikiClient,
            None,
            None,
            None,
        );

        let response = service
            .chat(
                501,
                KnowledgeAgentChatRequest {
                    tenant_id: 20001,
                    actor_id: None,
                    message: "What is the RAG boundary?".to_string(),
                    mode: Some(KnowledgeAgentKnowledgeMode::LlmWiki),
                    session_id: Some("session.1".to_string()),
                    model_provider_id: Some(CONTRACT_MODEL_PROVIDER_ID.to_string()),
                    model_id: Some("contract.default".to_string()),
                },
            )
            .await
            .expect("llm-wiki chat succeeds");

        assert_eq!(response.mode, KnowledgeAgentKnowledgeMode::LlmWiki);
        assert_eq!(response.model_provider_id, CONTRACT_MODEL_PROVIDER_ID);
        assert_eq!(response.citations.len(), 1);
        assert_eq!(
            response.citations[0].logical_path.as_deref(),
            Some("wiki/rag-boundary.md")
        );
        assert!(response.answer.contains("RAG Boundary"));
    }

    #[tokio::test]
    async fn chat_rag_mode_returns_chunk_citations() {
        let service = KnowledgeAgentChatService::new(
            &FakeProfileStore,
            &FakeRetrieval,
            FakeRetrievalClient,
            FakeWikiClient,
            None,
            None,
            None,
        );

        let response = service
            .chat(
                501,
                KnowledgeAgentChatRequest {
                    tenant_id: 20001,
                    actor_id: None,
                    message: "Explain hybrid retrieval".to_string(),
                    mode: Some(KnowledgeAgentKnowledgeMode::Rag),
                    session_id: None,
                    model_provider_id: Some(CONTRACT_MODEL_PROVIDER_ID.to_string()),
                    model_id: None,
                },
            )
            .await
            .expect("rag chat succeeds");

        assert_eq!(response.mode, KnowledgeAgentKnowledgeMode::Rag);
        assert_eq!(response.retrieval_id, Some(101));
        assert_eq!(response.citations[0].document_id, Some(301));
    }

    struct FakeProfileStore;

    #[async_trait]
    impl KnowledgeAgentProfileStore for FakeProfileStore {
        async fn create_profile(
            &self,
            _request: KnowledgeAgentProfileRequest,
        ) -> Result<KnowledgeAgentProfile, KnowledgeAgentProfileStoreError> {
            unimplemented!()
        }

        async fn retrieve_profile(
            &self,
            profile_id: u64,
        ) -> Result<KnowledgeAgentProfile, KnowledgeAgentProfileStoreError> {
            Ok(KnowledgeAgentProfile {
                profile_id,
                tenant_id: 20001,
                name: "Support Agent".to_string(),
                description: None,
                system_instruction: "Answer with citations.".to_string(),
                model_provider_id: DEFAULT_MODEL_PROVIDER_ID.to_string(),
                model_id: DEFAULT_MODEL_ID.to_string(),
                model_parameters: None,
                retrieval_profile_id: Some(31),
                citation_policy: None,
                memory_policy_ref: None,
                tool_policy_ref: None,
                answer_policy: None,
                knowledge_mode: KnowledgeAgentKnowledgeMode::LlmWiki,
                status: KnowledgeAgentStatus::Active,
                bindings: vec![KnowledgeAgentBinding {
                    binding_id: 601,
                    profile_id,
                    tenant_id: 20001,
                    space_id: 7,
                    collection_id: None,
                    source_filter: None,
                    document_filter: None,
                    priority: 0,
                    top_k: Some(4),
                    min_score: None,
                    enabled: true,
                }],
            })
        }

        async fn update_profile(
            &self,
            _profile_id: u64,
            _request: KnowledgeAgentProfileRequest,
        ) -> Result<KnowledgeAgentProfile, KnowledgeAgentProfileStoreError> {
            unimplemented!()
        }

        async fn delete_profile(
            &self,
            _profile_id: u64,
        ) -> Result<(), KnowledgeAgentProfileStoreError> {
            unimplemented!()
        }

        async fn list_bindings(
            &self,
            _profile_id: u64,
        ) -> Result<Vec<KnowledgeAgentBinding>, KnowledgeAgentProfileStoreError> {
            unimplemented!()
        }

        async fn create_binding(
            &self,
            _request: sdkwork_knowledgebase_contract::rag::KnowledgeAgentBindingRequest,
        ) -> Result<KnowledgeAgentBinding, KnowledgeAgentProfileStoreError> {
            unimplemented!()
        }

        async fn update_binding(
            &self,
            _profile_id: u64,
            _binding_id: u64,
            _request: sdkwork_knowledgebase_contract::rag::KnowledgeAgentBindingRequest,
        ) -> Result<KnowledgeAgentBinding, KnowledgeAgentProfileStoreError> {
            unimplemented!()
        }

        async fn delete_binding(
            &self,
            _profile_id: u64,
            _binding_id: u64,
        ) -> Result<(), KnowledgeAgentProfileStoreError> {
            unimplemented!()
        }
    }

    struct FakeRetrieval;

    #[async_trait]
    impl KnowledgeRetrievalExecutor for FakeRetrieval {
        async fn retrieve(
            &self,
            request: KnowledgeRetrievalRequest,
        ) -> Result<KnowledgeRetrievalResult, crate::retrieval::KnowledgeRetrievalServiceError>
        {
            assert_eq!(request.query, "Explain hybrid retrieval");
            Ok(KnowledgeRetrievalResult {
                retrieval_id: 101,
                trace: Some(KnowledgeRetrievalTrace {
                    retrieval_trace_id: 103,
                    status: "succeeded".to_string(),
                    latency_ms: Some(12),
                    result_count: 1,
                }),
                hits: vec![KnowledgeContextFragment {
                    chunk_id: 201,
                    document_id: 301,
                    document_version_id: None,
                    space_id: 7,
                    collection_id: None,
                    title: "Hybrid Retrieval".to_string(),
                    content: "Hybrid retrieval combines keyword and vector search.".to_string(),
                    score: Some(0.88),
                    rank: 1,
                    token_count: Some(10),
                    retrieval_method: KnowledgeRetrievalMethod::Hybrid,
                    citation: None,
                }],
            })
        }
    }

    #[derive(Clone)]
    struct FakeRetrievalClient;

    impl KnowledgebaseRetrievalClient for FakeRetrievalClient {
        fn retrieve(
            &self,
            _request: KnowledgeRetrievalRequest,
        ) -> Result<KnowledgeRetrievalResult, String> {
            Ok(KnowledgeRetrievalResult {
                retrieval_id: 101,
                trace: None,
                hits: vec![KnowledgeContextFragment {
                    chunk_id: 201,
                    document_id: 301,
                    document_version_id: None,
                    space_id: 7,
                    collection_id: None,
                    title: "Hybrid Retrieval".to_string(),
                    content: "Hybrid retrieval combines keyword and vector search.".to_string(),
                    score: Some(0.88),
                    rank: 1,
                    token_count: Some(10),
                    retrieval_method: KnowledgeRetrievalMethod::Hybrid,
                    citation: None,
                }],
            })
        }

        fn read_document(&self, document_id: &str) -> Result<KnowledgeDocument, String> {
            Ok(KnowledgeDocument::new(
                document_id,
                KnowledgeDocumentKind::Spec,
                "Hybrid Retrieval",
                "content",
            ))
        }

        fn list_documents(
            &self,
            _filter: KnowledgeDocumentFilter,
        ) -> Result<Vec<KnowledgeDocument>, String> {
            Ok(Vec::new())
        }
    }

    #[derive(Clone)]
    struct FakeWikiClient;

    impl LlmWikiKnowledgeClient for FakeWikiClient {
        fn search_wiki_pages(
            &self,
            space_id: u64,
            query: &str,
            top_k: usize,
        ) -> Result<Vec<WikiPageSummary>, String> {
            assert_eq!(space_id, 7);
            assert_eq!(query, "What is the RAG boundary?");
            assert_eq!(top_k, 4);
            Ok(vec![WikiPageSummary {
                title: "RAG Boundary".to_string(),
                slug: "rag-boundary".to_string(),
                page_type: WikiPageType::Concept,
                logical_path: "wiki/rag-boundary.md".to_string(),
                summary: "Knowledge retrieval is separate from model generation.".to_string(),
                source_count: 2,
                updated_at: "2026-06-01T00:00:00Z".to_string(),
                tags: vec!["rag".to_string()],
            }])
        }

        fn read_wiki_page_content(
            &self,
            _space_id: u64,
            _logical_path: &str,
        ) -> Result<String, String> {
            Ok("wiki page".to_string())
        }
    }
}
