use sdkwork_agent_kernel::{
    KernelError, KernelResult, KnowledgeDocumentKind, KnowledgeRetrievalMethod,
    KnowledgeSearchResult, RedactionClassification, TrustLevel,
};
use sdkwork_knowledgebase_contract::KnowledgeContextFragment;

pub(crate) fn parse_tenant_id(value: Option<&str>) -> Option<u64> {
    value.and_then(|value| value.parse::<u64>().ok())
}

pub(crate) fn parse_namespace_space_id(value: Option<&str>) -> KernelResult<u64> {
    let Some(value) = value else {
        return Err(KernelError::validation(
            "knowledgebase provider requires namespace to contain a knowledge space id",
        ));
    };

    let value = value.strip_prefix("space:").unwrap_or(value);
    value
        .parse::<u64>()
        .map_err(|_| KernelError::validation("knowledge namespace must be a numeric space id"))
}

pub(crate) fn map_method(
    method: KnowledgeRetrievalMethod,
) -> sdkwork_knowledgebase_contract::KnowledgeRetrievalMethod {
    match method {
        KnowledgeRetrievalMethod::Exact => {
            sdkwork_knowledgebase_contract::KnowledgeRetrievalMethod::Exact
        }
        KnowledgeRetrievalMethod::Keyword => {
            sdkwork_knowledgebase_contract::KnowledgeRetrievalMethod::Keyword
        }
        KnowledgeRetrievalMethod::FullText => {
            sdkwork_knowledgebase_contract::KnowledgeRetrievalMethod::FullText
        }
        KnowledgeRetrievalMethod::Structured => {
            sdkwork_knowledgebase_contract::KnowledgeRetrievalMethod::Structured
        }
        KnowledgeRetrievalMethod::Graph => {
            sdkwork_knowledgebase_contract::KnowledgeRetrievalMethod::Graph
        }
        KnowledgeRetrievalMethod::Vector => {
            sdkwork_knowledgebase_contract::KnowledgeRetrievalMethod::Vector
        }
        KnowledgeRetrievalMethod::Hybrid => {
            sdkwork_knowledgebase_contract::KnowledgeRetrievalMethod::Hybrid
        }
        KnowledgeRetrievalMethod::LlmRerank => {
            sdkwork_knowledgebase_contract::KnowledgeRetrievalMethod::LlmRerank
        }
        KnowledgeRetrievalMethod::External => {
            sdkwork_knowledgebase_contract::KnowledgeRetrievalMethod::External
        }
    }
}

pub(crate) fn hit_to_search_result(hit: KnowledgeContextFragment) -> KnowledgeSearchResult {
    let mut result = KnowledgeSearchResult::new(
        hit.chunk_id.to_string(),
        KnowledgeDocumentKind::Other,
        hit.title,
        map_contract_method(hit.retrieval_method),
    )
    .with_snippet(hit.content)
    .with_trust_level(TrustLevel::TrustedHost)
    .with_redaction_classification(RedactionClassification::TenantSensitive)
    .with_metadata("sdkwork.knowledge.chunk_id", hit.chunk_id.to_string())
    .with_metadata("sdkwork.knowledge.document_id", hit.document_id.to_string())
    .with_metadata("sdkwork.knowledge.space_id", hit.space_id.to_string())
    .with_metadata("sdkwork.knowledge.rank", hit.rank.to_string());

    if let Some(score) = hit.score {
        result = result.with_score(score);
    }

    if let Some(document_version_id) = hit.document_version_id {
        result = result.with_metadata(
            "sdkwork.knowledge.document_version_id",
            (document_version_id as u64).to_string(),
        );
    }

    if let Some(collection_id) = hit.collection_id {
        result = result.with_metadata(
            "sdkwork.knowledge.collection_id",
            (collection_id as u64).to_string(),
        );
    }

    if let Some(citation) = hit.citation {
        if let Some(source_uri) = citation.source_uri {
            result = result.with_source_uri(source_uri);
        }
        if let Some(locator) = citation.locator {
            result = result.with_metadata("sdkwork.knowledge.citation.locator", locator);
        }
    }

    result
}

fn map_contract_method(
    method: sdkwork_knowledgebase_contract::KnowledgeRetrievalMethod,
) -> KnowledgeRetrievalMethod {
    match method {
        sdkwork_knowledgebase_contract::KnowledgeRetrievalMethod::Exact => {
            KnowledgeRetrievalMethod::Exact
        }
        sdkwork_knowledgebase_contract::KnowledgeRetrievalMethod::Keyword => {
            KnowledgeRetrievalMethod::Keyword
        }
        sdkwork_knowledgebase_contract::KnowledgeRetrievalMethod::FullText => {
            KnowledgeRetrievalMethod::FullText
        }
        sdkwork_knowledgebase_contract::KnowledgeRetrievalMethod::Structured => {
            KnowledgeRetrievalMethod::Structured
        }
        sdkwork_knowledgebase_contract::KnowledgeRetrievalMethod::Graph => {
            KnowledgeRetrievalMethod::Graph
        }
        sdkwork_knowledgebase_contract::KnowledgeRetrievalMethod::Vector => {
            KnowledgeRetrievalMethod::Vector
        }
        sdkwork_knowledgebase_contract::KnowledgeRetrievalMethod::Hybrid => {
            KnowledgeRetrievalMethod::Hybrid
        }
        sdkwork_knowledgebase_contract::KnowledgeRetrievalMethod::LlmRerank => {
            KnowledgeRetrievalMethod::LlmRerank
        }
        sdkwork_knowledgebase_contract::KnowledgeRetrievalMethod::External => {
            KnowledgeRetrievalMethod::External
        }
    }
}
