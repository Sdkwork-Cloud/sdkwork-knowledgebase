use sdkwork_agent_kernel::{
    KernelError, KernelResult, KnowledgeDocument, KnowledgeDocumentFilter, KnowledgeDocumentKind,
    KnowledgeProvider, KnowledgeRetrievalMethod, KnowledgeSearchRequest, KnowledgeSearchResult,
    ProviderHealth, ProviderManifest, RedactionClassification, TrustLevel,
};
use sdkwork_knowledgebase_contract::wiki::WikiPageSummary;

pub const LLM_WIKI_KNOWLEDGE_PROVIDER_ID: &str = "provider.knowledge.llm-wiki";

pub trait LlmWikiKnowledgeClient {
    fn search_wiki_pages(
        &self,
        space_id: u64,
        query: &str,
        top_k: usize,
    ) -> Result<Vec<WikiPageSummary>, String>;

    fn read_wiki_page_content(&self, space_id: u64, logical_path: &str) -> Result<String, String>;
}

pub struct LlmWikiKnowledgeProvider<C> {
    client: C,
}

impl<C> LlmWikiKnowledgeProvider<C> {
    pub fn new(client: C) -> Self {
        Self { client }
    }
}

impl<C> KnowledgeProvider for LlmWikiKnowledgeProvider<C>
where
    C: LlmWikiKnowledgeClient,
{
    fn provider_manifest(&self) -> ProviderManifest {
        ProviderManifest::new(
            LLM_WIKI_KNOWLEDGE_PROVIDER_ID,
            "knowledge",
            "llm-wiki",
            env!("CARGO_PKG_VERSION"),
            vec![
                "knowledge.search".to_string(),
                "knowledge.read".to_string(),
                "knowledge.list".to_string(),
            ],
        )
    }

    fn search(&self, request: KnowledgeSearchRequest) -> KernelResult<Vec<KnowledgeSearchResult>> {
        if request.query.trim().is_empty() {
            return Err(KernelError::validation(
                "llm-wiki knowledge search query must not be blank",
            ));
        }

        let space_id = parse_namespace_space_id(request.namespace.as_deref())?;
        let top_k = request.top_k.max(1);
        let pages = self
            .client
            .search_wiki_pages(space_id, &request.query, top_k)
            .map_err(|message| {
                KernelError::provider_error("llm_wiki.search_failed", message)
                    .with_provider(LLM_WIKI_KNOWLEDGE_PROVIDER_ID)
            })?;

        Ok(pages
            .into_iter()
            .enumerate()
            .map(|(index, page)| wiki_page_to_search_result(space_id, index, page))
            .collect())
    }

    fn read(&self, document_id: &str) -> KernelResult<KnowledgeDocument> {
        let (space_id, logical_path) = parse_wiki_document_id(document_id)?;
        let content = self
            .client
            .read_wiki_page_content(space_id, &logical_path)
            .map_err(|message| {
                KernelError::provider_error("llm_wiki.read_failed", message)
                    .with_provider(LLM_WIKI_KNOWLEDGE_PROVIDER_ID)
            })?;

        Ok(KnowledgeDocument::new(
            document_id,
            KnowledgeDocumentKind::WikiPage,
            logical_path.clone(),
            content,
        )
        .with_namespace(format!("space:{space_id}")))
    }

    fn list(&self, filter: KnowledgeDocumentFilter) -> KernelResult<Vec<KnowledgeDocument>> {
        let space_id = parse_namespace_space_id(filter.namespace.as_deref())?;
        let pages = self
            .client
            .search_wiki_pages(space_id, "", usize::MAX)
            .map_err(|message| {
                KernelError::provider_error("llm_wiki.list_failed", message)
                    .with_provider(LLM_WIKI_KNOWLEDGE_PROVIDER_ID)
            })?;

        Ok(pages
            .into_iter()
            .map(|page| {
                KnowledgeDocument::new(
                    wiki_document_id(space_id, &page.logical_path),
                    KnowledgeDocumentKind::WikiPage,
                    page.title,
                    page.summary,
                )
                .with_namespace(format!("space:{space_id}"))
            })
            .collect())
    }

    fn health(&self) -> ProviderHealth {
        ProviderHealth::available()
    }
}

fn wiki_page_to_search_result(
    space_id: u64,
    index: usize,
    page: WikiPageSummary,
) -> KnowledgeSearchResult {
    let score = score_wiki_page(&page);
    let title = page.title.clone();
    KnowledgeSearchResult::new(
        wiki_document_id(space_id, &page.logical_path),
        KnowledgeDocumentKind::WikiPage,
        title.clone(),
        KnowledgeRetrievalMethod::Keyword,
    )
    .with_snippet(page.summary)
    .with_score(score)
    .with_source_uri(page.logical_path.clone())
    .with_trust_level(TrustLevel::TrustedHost)
    .with_redaction_classification(RedactionClassification::TenantSensitive)
    .with_metadata("sdkwork.knowledge.space_id", space_id.to_string())
    .with_metadata("sdkwork.knowledge.logical_path", page.logical_path)
    .with_metadata("sdkwork.knowledge.slug", page.slug)
    .with_metadata("sdkwork.knowledge.rank", (index + 1).to_string())
    .with_metadata("sdkwork.knowledge.title", title)
}

fn score_wiki_page(page: &WikiPageSummary) -> f64 {
    let tag_bonus = page.tags.len() as f64 * 0.01;
    let source_bonus = page.source_count as f64 * 0.02;
    0.5 + tag_bonus + source_bonus
}

pub fn wiki_document_id(space_id: u64, logical_path: &str) -> String {
    format!("wiki:{space_id}:{logical_path}")
}

fn parse_wiki_document_id(document_id: &str) -> KernelResult<(u64, String)> {
    let rest = document_id
        .strip_prefix("wiki:")
        .ok_or_else(|| KernelError::validation("llm-wiki document id must start with wiki:"))?;
    let (space_id, logical_path) = rest
        .split_once(':')
        .ok_or_else(|| KernelError::validation("llm-wiki document id must include space id"))?;
    let space_id = space_id
        .parse::<u64>()
        .map_err(|_| KernelError::validation("llm-wiki document id space id must be numeric"))?;
    if logical_path.trim().is_empty() {
        return Err(KernelError::validation(
            "llm-wiki document id logical path must not be blank",
        ));
    }
    Ok((space_id, logical_path.to_string()))
}

fn parse_namespace_space_id(namespace: Option<&str>) -> KernelResult<u64> {
    let namespace = namespace
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| {
            KernelError::validation("llm-wiki knowledge namespace must include space id")
        })?;
    let space_id = namespace
        .strip_prefix("space:")
        .unwrap_or(namespace)
        .parse::<u64>()
        .map_err(|_| {
            KernelError::validation("llm-wiki knowledge namespace space id must be numeric")
        })?;
    if space_id == 0 {
        return Err(KernelError::validation(
            "llm-wiki knowledge namespace space id must be positive",
        ));
    }
    Ok(space_id)
}

pub fn citations_from_rag_hits(
    hits: &[sdkwork_knowledgebase_contract::KnowledgeContextFragment],
) -> Vec<sdkwork_knowledgebase_contract::KnowledgeAgentChatCitation> {
    hits.iter()
        .map(
            |hit| sdkwork_knowledgebase_contract::KnowledgeAgentChatCitation {
                document_id: Some(hit.document_id),
                wiki_page_id: None,
                title: hit.title.clone(),
                source_uri: hit
                    .citation
                    .as_ref()
                    .and_then(|citation| citation.source_uri.clone()),
                logical_path: None,
                locator: hit
                    .citation
                    .as_ref()
                    .and_then(|citation| citation.locator.clone()),
                score: hit.score,
                snippet: Some(hit.content.clone()),
            },
        )
        .collect()
}

pub fn citations_from_wiki_pages(
    space_id: u64,
    pages: &[WikiPageSummary],
) -> Vec<sdkwork_knowledgebase_contract::KnowledgeAgentChatCitation> {
    pages
        .iter()
        .map(
            |page| sdkwork_knowledgebase_contract::KnowledgeAgentChatCitation {
                document_id: None,
                wiki_page_id: None,
                title: page.title.clone(),
                source_uri: Some(page.logical_path.clone()),
                logical_path: Some(page.logical_path.clone()),
                locator: Some(format!("space:{space_id}")),
                score: Some(score_wiki_page(page)),
                snippet: Some(page.summary.clone()),
            },
        )
        .collect()
}
