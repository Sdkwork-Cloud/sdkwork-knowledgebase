use sdkwork_agent_kernel::KnowledgeRetrievalMethod as KernelKnowledgeRetrievalMethod;
use sdkwork_knowledgebase_contract::agent_chat::{
    KnowledgeAgentChatCitation, KnowledgeAgentKnowledgeMode,
};
use sdkwork_knowledgebase_contract::rag::{
    KnowledgeAgentBinding, KnowledgeRetrievalBinding, KnowledgeRetrievalMethod,
    KnowledgeRetrievalRequest,
};
use std::sync::Arc;

use crate::{
    citations_from_engine_hits, citations_from_okf_concepts, citations_from_rag_hits,
    kernel_methods_for_retrieval, merge_retrieval_plan, KnowledgeRetrievalPlan, OkfKnowledgeClient,
};

#[derive(Debug, Clone)]
pub struct KnowledgeAccessRequest<'a> {
    pub tenant_id: u64,
    pub message: &'a str,
    pub mode: KnowledgeAgentKnowledgeMode,
    pub bindings: &'a [KnowledgeRetrievalBinding],
    pub top_k: usize,
    pub retrieval_profile_id: Option<u64>,
    pub retrieval_methods: Vec<KnowledgeRetrievalMethod>,
    pub retrieval_plan: Option<KnowledgeRetrievalPlan>,
}

#[derive(Debug, Clone)]
pub struct KnowledgeAccessResult {
    pub citations: Vec<KnowledgeAgentChatCitation>,
    pub retrieval_id: Option<u64>,
    pub namespace: String,
    pub kernel_methods: Vec<KernelKnowledgeRetrievalMethod>,
    pub resolved_knowledge_provider_id: Option<String>,
}

pub struct KnowledgeAccessGateway<O, E> {
    okf_client: O,
    retrieval_executor: E,
    space_engine: Option<Arc<dyn SpaceKnowledgeEngineClient>>,
}

impl<O, E> KnowledgeAccessGateway<O, E> {
    pub fn new(okf_client: O, retrieval_executor: E) -> Self {
        Self {
            okf_client,
            retrieval_executor,
            space_engine: None,
        }
    }

    pub fn with_space_engine(mut self, space_engine: Arc<dyn SpaceKnowledgeEngineClient>) -> Self {
        self.space_engine = Some(space_engine);
        self
    }
}

impl<O, E> KnowledgeAccessGateway<O, E>
where
    O: OkfKnowledgeClient + Clone,
    E: KnowledgeAccessRetrievalExecutor,
{
    pub async fn fetch(
        &self,
        request: KnowledgeAccessRequest<'_>,
    ) -> Result<KnowledgeAccessResult, String> {
        if request.bindings.is_empty() {
            return Err("knowledge access requires at least one binding".to_string());
        }

        let primary_space_id = request.bindings[0].space_id;
        let namespace = format!("space:{primary_space_id}");

        match request.mode {
            KnowledgeAgentKnowledgeMode::Rag => {
                let plan = request.retrieval_plan.clone().unwrap_or_else(|| {
                    merge_retrieval_plan(
                        &request.retrieval_methods,
                        None,
                        None,
                        None,
                        request.top_k,
                    )
                });
                let methods = plan.methods.clone();
                let retrieval_top_k = plan.top_k.unwrap_or(request.top_k as u32);

                let retrieval = self
                    .retrieval_executor
                    .retrieve(KnowledgeRetrievalRequest {
                        tenant_id: request.tenant_id,
                        actor_id: None,
                        query: request.message.to_string(),
                        retrieval_profile_id: request.retrieval_profile_id,
                        bindings: request.bindings.to_vec(),
                        methods,
                        top_k: Some(retrieval_top_k),
                        include_citations: true,
                        include_trace: true,
                        context_budget_tokens: None,
                        metadata: vec![],
                    })
                    .await?;

                let kernel_methods = kernel_methods_for_retrieval(&plan.methods);

                Ok(KnowledgeAccessResult {
                    citations: citations_from_rag_hits(&retrieval.hits),
                    retrieval_id: Some(retrieval.retrieval_id),
                    namespace,
                    kernel_methods,
                    resolved_knowledge_provider_id: None,
                })
            }
            KnowledgeAgentKnowledgeMode::OkfBundle => {
                let concepts = self.okf_client.search_okf_concepts(
                    primary_space_id,
                    request.message,
                    request.top_k,
                )?;

                Ok(KnowledgeAccessResult {
                    citations: citations_from_okf_concepts(primary_space_id, &concepts),
                    retrieval_id: None,
                    namespace,
                    kernel_methods: vec![KernelKnowledgeRetrievalMethod::Keyword],
                    resolved_knowledge_provider_id: None,
                })
            }
            KnowledgeAgentKnowledgeMode::External => {
                let client = self.space_engine.as_ref().ok_or_else(|| {
                    "external knowledge mode requires space engine client wiring".to_string()
                })?;
                let search = client
                    .search_space(
                        request.tenant_id,
                        primary_space_id,
                        request.message,
                        request.top_k as u32,
                    )
                    .await?;
                let provider_id = client.agent_provider_id_for_space(primary_space_id).await?;

                Ok(KnowledgeAccessResult {
                    citations: citations_from_engine_hits(primary_space_id, &search.hits),
                    retrieval_id: None,
                    namespace,
                    kernel_methods: vec![KernelKnowledgeRetrievalMethod::Hybrid],
                    resolved_knowledge_provider_id: Some(provider_id),
                })
            }
        }
    }
}

#[async_trait::async_trait]
pub trait SpaceKnowledgeEngineClient: Send + Sync {
    async fn search_space(
        &self,
        tenant_id: u64,
        space_id: u64,
        query: &str,
        top_k: u32,
    ) -> Result<sdkwork_knowledgebase_contract::knowledge_engine::KnowledgeEngineSearchResult, String>;

    async fn read_space_document(
        &self,
        tenant_id: u64,
        space_id: u64,
        scoped_document_id: &str,
    ) -> Result<sdkwork_knowledgebase_contract::knowledge_engine::KnowledgeEngineDocument, String>;

    async fn agent_provider_id_for_space(&self, space_id: u64) -> Result<String, String>;
}

#[async_trait::async_trait]
pub trait KnowledgeAccessRetrievalExecutor: Send + Sync {
    async fn retrieve(
        &self,
        request: KnowledgeRetrievalRequest,
    ) -> Result<sdkwork_knowledgebase_contract::KnowledgeRetrievalResult, String>;
}

#[async_trait::async_trait]
pub trait KnowledgeRetrievalPlanResolver: Send + Sync {
    async fn resolve_plan(
        &self,
        tenant_id: u64,
        retrieval_profile_id: Option<u64>,
    ) -> Result<Option<KnowledgeRetrievalPlan>, String>;
}

pub fn enabled_bindings(bindings: &[KnowledgeAgentBinding]) -> Vec<KnowledgeRetrievalBinding> {
    bindings
        .iter()
        .filter(|binding| binding.enabled)
        .map(|binding| KnowledgeRetrievalBinding {
            space_id: binding.space_id,
            collection_id: binding.collection_id,
            source_filter: binding.source_filter.clone(),
            document_filter: binding.document_filter.clone(),
            priority: binding.priority,
            top_k: binding.top_k,
            min_score: binding.min_score,
        })
        .collect()
}

pub fn default_top_k(bindings: &[KnowledgeRetrievalBinding]) -> usize {
    bindings
        .iter()
        .filter_map(|binding| binding.top_k)
        .map(|value| value as usize)
        .max()
        .unwrap_or(8)
        .clamp(1, 32)
}

pub fn resolve_chat_knowledge_mode(
    request_mode: Option<KnowledgeAgentKnowledgeMode>,
    profile_mode: KnowledgeAgentKnowledgeMode,
) -> KnowledgeAgentKnowledgeMode {
    request_mode.unwrap_or(profile_mode)
}

#[async_trait::async_trait]
pub trait KnowledgeSpaceModeResolver: Send + Sync {
    async fn knowledge_mode_for_space(
        &self,
        space_id: u64,
    ) -> Result<KnowledgeAgentKnowledgeMode, String>;
}

pub async fn validate_bindings_support_mode(
    resolver: &dyn KnowledgeSpaceModeResolver,
    bindings: &[KnowledgeRetrievalBinding],
    mode: KnowledgeAgentKnowledgeMode,
) -> Result<(), String> {
    for binding in bindings {
        let space_mode = resolver.knowledge_mode_for_space(binding.space_id).await?;
        if !space_supports_mode(space_mode, mode) {
            return Err(format!(
                "space {} is configured for {space_mode:?} but chat requested {mode:?}",
                binding.space_id
            ));
        }
    }
    Ok(())
}

pub fn space_supports_mode(
    space_mode: KnowledgeAgentKnowledgeMode,
    requested_mode: KnowledgeAgentKnowledgeMode,
) -> bool {
    space_mode == requested_mode
}

pub fn validate_rag_profile_requirements(
    mode: KnowledgeAgentKnowledgeMode,
    retrieval_profile_id: Option<u64>,
) -> Result<(), String> {
    if mode == KnowledgeAgentKnowledgeMode::Rag && retrieval_profile_id.is_none() {
        return Err("rag knowledge mode requires agent profile retrieval_profile_id".to_string());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use sdkwork_knowledgebase_contract::rag::KnowledgeContextFragment;

    #[derive(Clone)]
    struct FakeOkf;

    impl OkfKnowledgeClient for FakeOkf {
        fn search_okf_concepts(
            &self,
            _space_id: u64,
            _query: &str,
            _top_k: usize,
        ) -> Result<Vec<sdkwork_knowledgebase_contract::OkfConceptSummary>, String> {
            Ok(vec![])
        }

        fn read_okf_concept_content(
            &self,
            _space_id: u64,
            _logical_path: &str,
        ) -> Result<String, String> {
            Ok(String::new())
        }
    }

    #[derive(Clone)]
    struct FakeRetrieval;

    #[async_trait::async_trait]
    impl KnowledgeAccessRetrievalExecutor for FakeRetrieval {
        async fn retrieve(
            &self,
            request: KnowledgeRetrievalRequest,
        ) -> Result<sdkwork_knowledgebase_contract::KnowledgeRetrievalResult, String> {
            assert_eq!(request.methods, vec![KnowledgeRetrievalMethod::Hybrid]);
            Ok(sdkwork_knowledgebase_contract::KnowledgeRetrievalResult {
                retrieval_id: 42,
                trace: None,
                hits: vec![KnowledgeContextFragment {
                    chunk_id: 1,
                    document_id: 2,
                    document_version_id: None,
                    space_id: 7,
                    collection_id: None,
                    title: "Doc".to_string(),
                    content: "content".to_string(),
                    score: Some(0.9),
                    rank: 1,
                    token_count: Some(4),
                    retrieval_method: KnowledgeRetrievalMethod::Hybrid,
                    citation: None,
                }],
            })
        }
    }

    #[tokio::test]
    async fn rag_access_uses_retrieval_plan_methods() {
        let gateway = KnowledgeAccessGateway::new(FakeOkf, FakeRetrieval);
        let bindings = vec![KnowledgeRetrievalBinding {
            space_id: 7,
            collection_id: None,
            source_filter: None,
            document_filter: None,
            priority: 0,
            top_k: Some(4),
            min_score: None,
        }];

        let result = gateway
            .fetch(KnowledgeAccessRequest {
                tenant_id: 1,
                message: "query",
                mode: KnowledgeAgentKnowledgeMode::Rag,
                bindings: &bindings,
                top_k: 4,
                retrieval_profile_id: Some(10),
                retrieval_methods: vec![],
                retrieval_plan: Some(KnowledgeRetrievalPlan::default()),
            })
            .await
            .expect("rag fetch");

        assert_eq!(result.retrieval_id, Some(42));
        assert_eq!(result.citations.len(), 1);
    }

    #[tokio::test]
    async fn external_access_resolves_engine_provider_and_citations() {
        #[derive(Clone)]
        struct FakeSpaceEngine;

        #[async_trait::async_trait]
        impl SpaceKnowledgeEngineClient for FakeSpaceEngine {
            async fn search_space(
                &self,
                _tenant_id: u64,
                _space_id: u64,
                _query: &str,
                _top_k: u32,
            ) -> Result<
                sdkwork_knowledgebase_contract::knowledge_engine::KnowledgeEngineSearchResult,
                String,
            > {
                Ok(
                    sdkwork_knowledgebase_contract::knowledge_engine::KnowledgeEngineSearchResult {
                        implementation_id: "engine.knowledge.external.dify".to_string(),
                        hits: vec![
                            sdkwork_knowledgebase_contract::knowledge_engine::KnowledgeEngineSearchHit {
                                document:
                                    sdkwork_knowledgebase_contract::knowledge_engine::KnowledgeEngineDocumentRef {
                                        document_id: "doc-1".to_string(),
                                        title: "External Doc".to_string(),
                                        source_uri: Some("external://doc-1".to_string()),
                                    },
                                snippet: "external snippet".to_string(),
                                score: Some(0.8),
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

        let gateway = KnowledgeAccessGateway::new(FakeOkf, FakeRetrieval)
            .with_space_engine(Arc::new(FakeSpaceEngine));
        let bindings = vec![KnowledgeRetrievalBinding {
            space_id: 9,
            collection_id: None,
            source_filter: None,
            document_filter: None,
            priority: 0,
            top_k: Some(4),
            min_score: None,
        }];

        let result = gateway
            .fetch(KnowledgeAccessRequest {
                tenant_id: 1,
                message: "query",
                mode: KnowledgeAgentKnowledgeMode::External,
                bindings: &bindings,
                top_k: 4,
                retrieval_profile_id: None,
                retrieval_methods: vec![],
                retrieval_plan: None,
            })
            .await
            .expect("external fetch");

        assert_eq!(
            result.resolved_knowledge_provider_id.as_deref(),
            Some("provider.knowledge.external.dify")
        );
        assert_eq!(result.citations.len(), 1);
        assert_eq!(result.citations[0].title, "External Doc");
        assert_eq!(result.citations[0].concept_id.as_deref(), Some("doc-1"));
        assert_eq!(result.citations[0].logical_path.as_deref(), Some("9/doc-1"));
    }

    #[test]
    fn rag_mode_requires_retrieval_profile() {
        assert!(validate_rag_profile_requirements(KnowledgeAgentKnowledgeMode::Rag, None).is_err());
        assert!(
            validate_rag_profile_requirements(KnowledgeAgentKnowledgeMode::Rag, Some(1)).is_ok()
        );
    }
}
