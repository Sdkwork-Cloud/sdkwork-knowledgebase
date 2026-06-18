use sdkwork_agent_kernel::KnowledgeRetrievalMethod as KernelKnowledgeRetrievalMethod;
use sdkwork_knowledgebase_contract::agent_chat::{
    KnowledgeAgentChatCitation, KnowledgeAgentKnowledgeMode,
};
use sdkwork_knowledgebase_contract::rag::{
    KnowledgeAgentBinding, KnowledgeRetrievalBinding, KnowledgeRetrievalMethod,
    KnowledgeRetrievalRequest,
};

use crate::{
    citations_from_rag_hits, citations_from_wiki_pages, kernel_methods_for_retrieval,
    merge_retrieval_plan, KnowledgeRetrievalPlan, LlmWikiKnowledgeClient,
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
}

pub struct KnowledgeAccessGateway<W, E> {
    wiki_client: W,
    retrieval_executor: E,
}

impl<W, E> KnowledgeAccessGateway<W, E> {
    pub fn new(wiki_client: W, retrieval_executor: E) -> Self {
        Self {
            wiki_client,
            retrieval_executor,
        }
    }
}

impl<W, E> KnowledgeAccessGateway<W, E>
where
    W: LlmWikiKnowledgeClient + Clone,
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
                })
            }
            KnowledgeAgentKnowledgeMode::LlmWiki => {
                let pages = self.wiki_client.search_wiki_pages(
                    primary_space_id,
                    request.message,
                    request.top_k,
                )?;

                Ok(KnowledgeAccessResult {
                    citations: citations_from_wiki_pages(primary_space_id, &pages),
                    retrieval_id: None,
                    namespace,
                    kernel_methods: vec![KernelKnowledgeRetrievalMethod::Keyword],
                })
            }
        }
    }
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
    struct FakeWiki;

    impl LlmWikiKnowledgeClient for FakeWiki {
        fn search_wiki_pages(
            &self,
            _space_id: u64,
            _query: &str,
            _top_k: usize,
        ) -> Result<Vec<sdkwork_knowledgebase_contract::wiki::WikiPageSummary>, String> {
            Ok(vec![])
        }

        fn read_wiki_page_content(
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
        let gateway = KnowledgeAccessGateway::new(FakeWiki, FakeRetrieval);
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

    #[test]
    fn rag_mode_requires_retrieval_profile() {
        assert!(validate_rag_profile_requirements(KnowledgeAgentKnowledgeMode::Rag, None).is_err());
        assert!(
            validate_rag_profile_requirements(KnowledgeAgentKnowledgeMode::Rag, Some(1)).is_ok()
        );
    }
}
