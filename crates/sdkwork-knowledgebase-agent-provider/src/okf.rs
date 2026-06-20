use sdkwork_agent_kernel::{
    KernelError, KernelResult, KnowledgeDocument, KnowledgeDocumentFilter, KnowledgeDocumentKind,
    KnowledgeProvider, KnowledgeRetrievalMethod, KnowledgeSearchRequest, KnowledgeSearchResult,
    ProviderHealth, ProviderManifest, RedactionClassification, TrustLevel,
};
use sdkwork_knowledgebase_contract::{
    okf::OkfBundlePaths, OkfConceptSummary, OKF_KNOWLEDGE_PROVIDER_ID,
};

pub trait OkfKnowledgeClient {
    fn search_okf_concepts(
        &self,
        space_id: u64,
        query: &str,
        top_k: usize,
    ) -> Result<Vec<OkfConceptSummary>, String>;

    fn read_okf_concept_content(&self, space_id: u64, logical_path: &str)
        -> Result<String, String>;
}

pub struct OkfKnowledgeProvider<C> {
    client: C,
}

impl<C> OkfKnowledgeProvider<C> {
    pub fn new(client: C) -> Self {
        Self { client }
    }
}

impl<C> KnowledgeProvider for OkfKnowledgeProvider<C>
where
    C: OkfKnowledgeClient,
{
    fn provider_manifest(&self) -> ProviderManifest {
        ProviderManifest::new(
            OKF_KNOWLEDGE_PROVIDER_ID,
            "knowledge",
            "okf-bundle",
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
                "okf-bundle knowledge search query must not be blank",
            ));
        }

        let space_id = parse_namespace_space_id(request.namespace.as_deref())?;
        let top_k = request.top_k.max(1);
        let pages = self
            .client
            .search_okf_concepts(space_id, &request.query, top_k)
            .map_err(|message| {
                KernelError::provider_error("okf_bundle.search_failed", message)
                    .with_provider(OKF_KNOWLEDGE_PROVIDER_ID)
            })?;

        Ok(pages
            .into_iter()
            .enumerate()
            .map(|(index, page)| okf_concept_to_search_result(space_id, index, page))
            .collect())
    }

    fn read(&self, document_id: &str) -> KernelResult<KnowledgeDocument> {
        let (space_id, concept_id) = parse_okf_document_id(document_id)?;
        let logical_path = OkfBundlePaths::concept_logical_path(&concept_id);
        let content = self
            .client
            .read_okf_concept_content(space_id, &logical_path)
            .map_err(|message| {
                KernelError::provider_error("okf_bundle.read_failed", message)
                    .with_provider(OKF_KNOWLEDGE_PROVIDER_ID)
            })?;

        Ok(KnowledgeDocument::new(
            document_id,
            KnowledgeDocumentKind::WikiPage,
            concept_id.clone(),
            content,
        )
        .with_namespace(format!("space:{space_id}"))
        .with_metadata("sdkwork.knowledge.logical_path", logical_path))
    }

    fn list(&self, filter: KnowledgeDocumentFilter) -> KernelResult<Vec<KnowledgeDocument>> {
        let space_id = parse_namespace_space_id(filter.namespace.as_deref())?;
        let pages = self
            .client
            .search_okf_concepts(space_id, "", usize::MAX)
            .map_err(|message| {
                KernelError::provider_error("okf_bundle.list_failed", message)
                    .with_provider(OKF_KNOWLEDGE_PROVIDER_ID)
            })?;

        Ok(pages
            .into_iter()
            .map(|page| {
                KnowledgeDocument::new(
                    okf_document_id(space_id, &page.concept_id),
                    KnowledgeDocumentKind::WikiPage,
                    page.title.clone(),
                    page.description.clone(),
                )
                .with_namespace(format!("space:{space_id}"))
                .with_metadata("sdkwork.knowledge.logical_path", page.logical_path.clone())
                .with_metadata("sdkwork.knowledge.concept_id", page.concept_id.clone())
            })
            .collect())
    }

    fn health(&self) -> ProviderHealth {
        ProviderHealth::available()
    }
}

fn okf_concept_to_search_result(
    space_id: u64,
    index: usize,
    page: OkfConceptSummary,
) -> KnowledgeSearchResult {
    let score = score_okf_concept(&page);
    let title = page.title.clone();
    KnowledgeSearchResult::new(
        okf_document_id(space_id, &page.concept_id),
        KnowledgeDocumentKind::WikiPage,
        title.clone(),
        KnowledgeRetrievalMethod::Keyword,
    )
    .with_snippet(page.description)
    .with_score(score)
    .with_source_uri(page.logical_path.clone())
    .with_trust_level(TrustLevel::TrustedHost)
    .with_redaction_classification(RedactionClassification::TenantSensitive)
    .with_metadata("sdkwork.knowledge.space_id", space_id.to_string())
    .with_metadata("sdkwork.knowledge.logical_path", page.logical_path)
    .with_metadata("sdkwork.knowledge.concept_id", page.concept_id)
    .with_metadata("sdkwork.knowledge.rank", (index + 1).to_string())
    .with_metadata("sdkwork.knowledge.title", title)
}

fn score_okf_concept(page: &OkfConceptSummary) -> f64 {
    let tag_bonus = page.tags.len() as f64 * 0.01;
    let source_bonus = page.source_count as f64 * 0.02;
    0.5 + tag_bonus + source_bonus
}

fn parse_okf_document_id(document_id: &str) -> KernelResult<(u64, String)> {
    let rest = document_id
        .strip_prefix("okf:")
        .ok_or_else(|| KernelError::validation("okf-bundle document id must start with okf:"))?;
    let (space_id, concept_id) = rest
        .split_once(':')
        .ok_or_else(|| KernelError::validation("okf-bundle document id must include space id"))?;
    let space_id = space_id
        .parse::<u64>()
        .map_err(|_| KernelError::validation("okf-bundle document id space id must be numeric"))?;
    if concept_id.trim().is_empty() {
        return Err(KernelError::validation(
            "okf-bundle document id concept id must not be blank",
        ));
    }
    Ok((space_id, concept_id.to_string()))
}

fn parse_namespace_space_id(namespace: Option<&str>) -> KernelResult<u64> {
    let namespace = namespace
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| {
            KernelError::validation("okf-bundle knowledge namespace must include space id")
        })?;
    let space_id = namespace
        .strip_prefix("space:")
        .unwrap_or(namespace)
        .parse::<u64>()
        .map_err(|_| {
            KernelError::validation("okf-bundle knowledge namespace space id must be numeric")
        })?;
    if space_id == 0 {
        return Err(KernelError::validation(
            "okf-bundle knowledge namespace space id must be positive",
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
                okf_concept_id: None,
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

pub fn citations_from_okf_concepts(
    space_id: u64,
    concepts: &[OkfConceptSummary],
) -> Vec<sdkwork_knowledgebase_contract::KnowledgeAgentChatCitation> {
    concepts
        .iter()
        .map(
            |concept| sdkwork_knowledgebase_contract::KnowledgeAgentChatCitation {
                document_id: Some(okf_document_id(space_id, &concept.concept_id)),
                okf_concept_id: Some(concept.concept_id.clone()),
                title: concept.title.clone(),
                source_uri: Some(concept.logical_path.clone()),
                logical_path: Some(concept.logical_path.clone()),
                locator: Some(format!("space:{space_id}")),
                score: Some(score_okf_concept(concept)),
                snippet: Some(concept.description.clone()),
            },
        )
        .collect()
}
