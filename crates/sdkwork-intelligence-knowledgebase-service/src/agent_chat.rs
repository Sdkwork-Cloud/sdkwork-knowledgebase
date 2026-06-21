use crate::ports::knowledge_agent_profile_store::{
    KnowledgeAgentProfileStore, KnowledgeAgentProfileStoreError,
};
use crate::retrieval::{KnowledgeRetrievalExecutor, KnowledgeRetrievalServiceError};
use sdkwork_agent_kernel::{AgentChatRequest, AgentChatService, KernelError};
use sdkwork_agent_plugin_rig::ids as rig_ids;
use sdkwork_knowledgebase_agent_provider::{
    build_knowledge_agent_runtime, default_top_k, resolve_chat_knowledge_mode,
    resolve_model_provider_for_implementation, validate_bindings_support_mode,
    validate_rag_profile_requirements, validate_registered_agent_implementation,
    KnowledgeAccessGateway, KnowledgeAccessRequest, KnowledgeAccessRetrievalExecutor,
    KnowledgeAgentRuntimeBuildRequest, KnowledgeRetrievalPlanResolver, KnowledgeSpaceModeResolver,
    KnowledgebaseRetrievalClient, OkfKnowledgeClient, SpaceKnowledgeEngineClient,
    OKF_KNOWLEDGE_PROVIDER_ID, SDKWORK_KNOWLEDGEBASE_PROVIDER_ID,
};
use sdkwork_knowledgebase_contract::agent_chat::{
    KnowledgeAgentChatRequest, KnowledgeAgentChatResponse, KnowledgeAgentKnowledgeMode,
};
use sdkwork_knowledgebase_contract::rag::KnowledgeRetrievalRequest;
use sdkwork_knowledgebase_contract::resolve_agent_implementation_id;
use sdkwork_utils_rust::is_blank;
use std::sync::Arc;
use thiserror::Error;
use uuid::Uuid;

pub const DEFAULT_MODEL_PROVIDER_ID: &str = rig_ids::MODEL_PROVIDER_ID;
pub const DEFAULT_MODEL_ID: &str = rig_ids::DEFAULT_MODEL_ID;
pub use sdkwork_knowledgebase_agent_provider::CONTRACT_MODEL_PROVIDER_ID;

pub struct KnowledgeAgentChatService<'a, R, W> {
    profiles: &'a dyn KnowledgeAgentProfileStore,
    retrieval: &'a dyn KnowledgeRetrievalExecutor,
    retrieval_client: R,
    okf_client: W,
    claw_router_client: Option<Arc<clawrouter_open_sdk::SdkworkAiClient>>,
    retrieval_plan_resolver: Option<&'a dyn KnowledgeRetrievalPlanResolver>,
    space_mode_resolver: Option<&'a dyn KnowledgeSpaceModeResolver>,
    space_engine_client: Option<Arc<dyn SpaceKnowledgeEngineClient>>,
}

impl<'a, R, W> KnowledgeAgentChatService<'a, R, W>
where
    R: KnowledgebaseRetrievalClient + Send + Sync + Clone + 'static,
    W: OkfKnowledgeClient + Send + Sync + Clone + 'static,
{
    pub fn new(
        profiles: &'a dyn KnowledgeAgentProfileStore,
        retrieval: &'a dyn KnowledgeRetrievalExecutor,
        retrieval_client: R,
        okf_client: W,
        claw_router_client: Option<Arc<clawrouter_open_sdk::SdkworkAiClient>>,
        retrieval_plan_resolver: Option<&'a dyn KnowledgeRetrievalPlanResolver>,
        space_mode_resolver: Option<&'a dyn KnowledgeSpaceModeResolver>,
        space_engine_client: Option<Arc<dyn SpaceKnowledgeEngineClient>>,
    ) -> Self {
        Self {
            profiles,
            retrieval,
            retrieval_client,
            okf_client,
            claw_router_client,
            retrieval_plan_resolver,
            space_mode_resolver,
            space_engine_client,
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
        if is_blank(Some(request.message.as_str())) {
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

        let access = {
            let gateway = KnowledgeAccessGateway::new(
                self.okf_client.clone(),
                RetrievalExecutorAdapter {
                    retrieval: self.retrieval,
                },
            );
            if let Some(space_engine) = self.space_engine_client.clone() {
                gateway.with_space_engine(space_engine)
            } else {
                gateway
            }
        };
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
            .filter(|value| !is_blank(Some(value.as_str())))
            .unwrap_or_else(|| profile.model_provider_id.clone());
        let agent_implementation_id = resolve_agent_implementation_id(
            request.agent_implementation_id.as_deref(),
            &profile.agent_implementation_id,
        );
        validate_registered_agent_implementation(&agent_implementation_id)
            .map_err(KnowledgeAgentChatServiceError::InvalidRequest)?;
        let resolved_model_provider_id =
            resolve_model_provider_for_implementation(&agent_implementation_id, &model_provider_id);
        let model_id = request
            .model_id
            .clone()
            .filter(|value| !is_blank(Some(value.as_str())))
            .unwrap_or_else(|| profile.model_id.clone());

        let chat_id = format!("chat.{}", Uuid::new_v4());
        let external_knowledge_provider_ids = access_result
            .resolved_knowledge_provider_id
            .clone()
            .into_iter()
            .collect::<Vec<_>>();
        let runtime = build_knowledge_agent_runtime(KnowledgeAgentRuntimeBuildRequest {
            agent_implementation_id: agent_implementation_id.clone(),
            model_provider_id: resolved_model_provider_id.clone(),
            mode,
            retrieval_client: self.retrieval_client.clone(),
            okf_client: self.okf_client.clone(),
            tenant_id: request.tenant_id,
            claw_router_client: self.claw_router_client.clone(),
            space_engine_client: self.space_engine_client.clone(),
            external_knowledge_provider_ids,
        })
        .map_err(KnowledgeAgentChatServiceError::Runtime)?;

        let knowledge_provider_id = access_result
            .resolved_knowledge_provider_id
            .as_deref()
            .unwrap_or_else(|| default_knowledge_provider_id(mode));

        let mut chat_request =
            AgentChatRequest::new(chat_id.clone(), vec![request.message.clone()])
                .with_provider_id(resolved_model_provider_id.clone())
                .with_model_id(model_id.clone())
                .with_knowledge_query(&request.message)
                .with_knowledge_provider_id(knowledge_provider_id)
                .with_knowledge_tenant_id(request.tenant_id.to_string())
                .with_knowledge_namespace(knowledge_namespace)
                .with_knowledge_top_k(default_top_k(&bindings));

        for method in knowledge_methods {
            chat_request = chat_request.with_knowledge_method(method);
        }

        if let Some(session_id) = request.session_id.clone() {
            chat_request = chat_request.for_session(session_id.clone());
        }

        if !is_blank(Some(profile.system_instruction.as_str())) {
            chat_request = chat_request.with_metadata(
                "sdkwork.knowledge.system_instruction",
                profile.system_instruction.clone(),
            );
        }

        let response = tokio::task::spawn_blocking({
            let chat_request = chat_request;
            move || AgentChatService::new().invoke(&runtime, chat_request)
        })
        .await
        .map_err(|error| {
            KnowledgeAgentChatServiceError::AgentKernel(format!(
                "agent chat worker join failed: {error}"
            ))
        })?
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
            agent_implementation_id,
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

fn default_knowledge_provider_id(mode: KnowledgeAgentKnowledgeMode) -> &'static str {
    match mode {
        KnowledgeAgentKnowledgeMode::OkfBundle => OKF_KNOWLEDGE_PROVIDER_ID,
        KnowledgeAgentKnowledgeMode::Rag => SDKWORK_KNOWLEDGEBASE_PROVIDER_ID,
        KnowledgeAgentKnowledgeMode::External => "provider.knowledge.external.unresolved",
    }
}

fn map_kernel_error(error: KernelError) -> KnowledgeAgentChatServiceError {
    KnowledgeAgentChatServiceError::AgentKernel(error.to_string())
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
    use sdkwork_knowledgebase_contract::okf::OkfConceptSummary;
    use sdkwork_knowledgebase_contract::rag::{
        KnowledgeAgentBinding, KnowledgeAgentProfile, KnowledgeAgentProfileRequest,
        KnowledgeAgentStatus, KnowledgeContextFragment, KnowledgeRetrievalMethod,
        KnowledgeRetrievalResult, KnowledgeRetrievalTrace,
    };
    use sdkwork_knowledgebase_contract::KNOWLEDGEBASE_CONTRACT_AGENT_IMPLEMENTATION_ID;

    #[tokio::test]
    async fn chat_defaults_to_okf_bundle_mode_with_citations_and_contract_model() {
        let service = KnowledgeAgentChatService::new(
            &FakeProfileStore,
            &FakeRetrieval,
            FakeRetrievalClient,
            FakeOkfClient,
            None,
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
                    mode: Some(KnowledgeAgentKnowledgeMode::OkfBundle),
                    session_id: Some("session.1".to_string()),
                    model_provider_id: Some(CONTRACT_MODEL_PROVIDER_ID.to_string()),
                    model_id: Some("contract.default".to_string()),
                    agent_implementation_id: Some(
                        KNOWLEDGEBASE_CONTRACT_AGENT_IMPLEMENTATION_ID.to_string(),
                    ),
                },
            )
            .await
            .expect("okf bundle chat succeeds");

        assert_eq!(response.mode, KnowledgeAgentKnowledgeMode::OkfBundle);
        assert_eq!(
            response.agent_implementation_id,
            KNOWLEDGEBASE_CONTRACT_AGENT_IMPLEMENTATION_ID
        );
        assert_eq!(response.model_provider_id, CONTRACT_MODEL_PROVIDER_ID);
        assert_eq!(response.citations.len(), 1);
        assert_eq!(
            response.citations[0].logical_path.as_deref(),
            Some("7/concepts/rag-boundary")
        );
        assert_eq!(
            response.citations[0].concept_id.as_deref(),
            Some("concepts/rag-boundary")
        );
        assert_eq!(
            response.citations[0].locator.as_deref(),
            Some("okf:7:concepts/rag-boundary")
        );
        assert!(response.answer.contains("RAG Boundary"));
    }

    #[tokio::test]
    async fn chat_rag_mode_returns_chunk_citations() {
        let service = KnowledgeAgentChatService::new(
            &FakeProfileStore,
            &FakeRetrieval,
            FakeRetrievalClient,
            FakeOkfClient,
            None,
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
                    agent_implementation_id: Some(
                        KNOWLEDGEBASE_CONTRACT_AGENT_IMPLEMENTATION_ID.to_string(),
                    ),
                },
            )
            .await
            .expect("rag chat succeeds");

        assert_eq!(response.mode, KnowledgeAgentKnowledgeMode::Rag);
        assert_eq!(response.retrieval_id, Some(101));
        assert_eq!(response.citations[0].document_id, Some(301));
    }

    #[tokio::test]
    async fn chat_external_mode_registers_engine_provider_and_returns_citations() {
        struct ExternalProfileStore;

        #[async_trait]
        impl KnowledgeAgentProfileStore for ExternalProfileStore {
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
                    name: "External Agent".to_string(),
                    description: None,
                    system_instruction: "Answer with citations.".to_string(),
                    model_provider_id: CONTRACT_MODEL_PROVIDER_ID.to_string(),
                    model_id: "contract".to_string(),
                    model_parameters: None,
                    retrieval_profile_id: None,
                    citation_policy: None,
                    memory_policy_ref: None,
                    tool_policy_ref: None,
                    answer_policy: None,
                    knowledge_mode: KnowledgeAgentKnowledgeMode::External,
                    agent_implementation_id: KNOWLEDGEBASE_CONTRACT_AGENT_IMPLEMENTATION_ID
                        .to_string(),
                    status: KnowledgeAgentStatus::Active,
                    bindings: vec![KnowledgeAgentBinding {
                        binding_id: 701,
                        profile_id,
                        tenant_id: 20001,
                        space_id: 9,
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

        #[derive(Clone)]
        struct FakeExternalSpaceModeResolver;

        #[async_trait]
        impl KnowledgeSpaceModeResolver for FakeExternalSpaceModeResolver {
            async fn knowledge_mode_for_space(
                &self,
                _space_id: u64,
            ) -> Result<KnowledgeAgentKnowledgeMode, String> {
                Ok(KnowledgeAgentKnowledgeMode::External)
            }
        }

        #[derive(Clone)]
        struct FakeSpaceEngine;

        #[async_trait]
        impl SpaceKnowledgeEngineClient for FakeSpaceEngine {
            async fn search_space(
                &self,
                _tenant_id: u64,
                space_id: u64,
                query: &str,
                top_k: u32,
            ) -> Result<
                sdkwork_knowledgebase_contract::knowledge_engine::KnowledgeEngineSearchResult,
                String,
            > {
                assert_eq!(space_id, 9);
                assert_eq!(query, "What is in the external knowledge base?");
                assert_eq!(top_k, 4);
                Ok(
                    sdkwork_knowledgebase_contract::knowledge_engine::KnowledgeEngineSearchResult {
                        implementation_id: "engine.knowledge.external.dify".to_string(),
                        hits: vec![
                            sdkwork_knowledgebase_contract::knowledge_engine::KnowledgeEngineSearchHit {
                                document:
                                    sdkwork_knowledgebase_contract::knowledge_engine::KnowledgeEngineDocumentRef {
                                        document_id: "9/seg-1".to_string(),
                                        title: "External Doc".to_string(),
                                        source_uri: Some("external://doc-1".to_string()),
                                    },
                                snippet: "external snippet".to_string(),
                                score: Some(0.91),
                            },
                        ],
                    },
                )
            }

            async fn agent_provider_id_for_space(&self, _space_id: u64) -> Result<String, String> {
                Ok("provider.knowledge.external.dify".to_string())
            }

            async fn read_space_document(
                &self,
                _tenant_id: u64,
                _space_id: u64,
                _scoped_document_id: &str,
            ) -> Result<
                sdkwork_knowledgebase_contract::knowledge_engine::KnowledgeEngineDocument,
                String,
            > {
                Err("not implemented".to_string())
            }
        }

        let service = KnowledgeAgentChatService::new(
            &ExternalProfileStore,
            &FakeRetrieval,
            FakeRetrievalClient,
            FakeOkfClient,
            None,
            None,
            Some(&FakeExternalSpaceModeResolver),
            Some(Arc::new(FakeSpaceEngine)),
        );

        let response = service
            .chat(
                801,
                KnowledgeAgentChatRequest {
                    tenant_id: 20001,
                    actor_id: None,
                    message: "What is in the external knowledge base?".to_string(),
                    mode: Some(KnowledgeAgentKnowledgeMode::External),
                    session_id: None,
                    model_provider_id: Some(CONTRACT_MODEL_PROVIDER_ID.to_string()),
                    model_id: Some("contract".to_string()),
                    agent_implementation_id: Some(
                        KNOWLEDGEBASE_CONTRACT_AGENT_IMPLEMENTATION_ID.to_string(),
                    ),
                },
            )
            .await
            .expect("external chat succeeds");

        assert_eq!(response.mode, KnowledgeAgentKnowledgeMode::External);
        assert_eq!(response.citations.len(), 1);
        assert_eq!(response.citations[0].title, "External Doc");
        assert_eq!(
            response.citations[0].logical_path.as_deref(),
            Some("9/seg-1")
        );
        assert!(response.answer.contains("External Doc"));
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
                knowledge_mode: KnowledgeAgentKnowledgeMode::OkfBundle,
                agent_implementation_id: KNOWLEDGEBASE_CONTRACT_AGENT_IMPLEMENTATION_ID.to_string(),
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
    struct FakeOkfClient;

    impl OkfKnowledgeClient for FakeOkfClient {
        fn search_okf_concepts(
            &self,
            space_id: u64,
            query: &str,
            top_k: usize,
        ) -> Result<Vec<OkfConceptSummary>, String> {
            assert_eq!(space_id, 7);
            assert_eq!(query, "What is the RAG boundary?");
            assert_eq!(top_k, 4);
            Ok(vec![OkfConceptSummary {
                title: "RAG Boundary".to_string(),
                concept_id: "concepts/rag-boundary".to_string(),
                concept_type: "Knowledge Concept".to_string(),
                logical_path: "okf/concepts/rag-boundary.md".to_string(),
                bundle_relative_path: "concepts/rag-boundary.md".to_string(),
                description: "Knowledge retrieval is separate from model generation.".to_string(),
                source_count: 2,
                updated_at: "2026-06-01T00:00:00Z".to_string(),
                tags: vec!["rag".to_string()],
            }])
        }

        fn read_okf_concept_content(
            &self,
            _space_id: u64,
            _logical_path: &str,
        ) -> Result<String, String> {
            Ok("okf concept".to_string())
        }
    }
}
