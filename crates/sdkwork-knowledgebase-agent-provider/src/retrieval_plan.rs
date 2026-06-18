use sdkwork_knowledgebase_contract::rag::KnowledgeRetrievalMethod;

#[derive(Debug, Clone, PartialEq)]
pub struct KnowledgeRetrievalPlan {
    pub methods: Vec<KnowledgeRetrievalMethod>,
    pub top_k: Option<u32>,
    pub min_score: Option<f64>,
}

impl Default for KnowledgeRetrievalPlan {
    fn default() -> Self {
        Self {
            methods: default_rag_methods(),
            top_k: None,
            min_score: None,
        }
    }
}

pub fn default_rag_methods() -> Vec<KnowledgeRetrievalMethod> {
    vec![KnowledgeRetrievalMethod::Hybrid]
}

pub fn retrieval_methods_for_strategy(strategy: &str) -> Vec<KnowledgeRetrievalMethod> {
    match strategy.trim().to_ascii_lowercase().as_str() {
        "keyword" | "lexical" => vec![KnowledgeRetrievalMethod::Keyword],
        "vector" | "semantic" | "embedding" => vec![KnowledgeRetrievalMethod::Vector],
        "hybrid" | "default" | "" => default_rag_methods(),
        "full_text" | "fulltext" => vec![KnowledgeRetrievalMethod::FullText],
        other if other.contains("hybrid") => default_rag_methods(),
        other if other.contains("vector") => vec![KnowledgeRetrievalMethod::Vector],
        other if other.contains("keyword") => vec![KnowledgeRetrievalMethod::Keyword],
        _ => default_rag_methods(),
    }
}

pub fn merge_retrieval_plan(
    request_methods: &[KnowledgeRetrievalMethod],
    profile_strategy: Option<&str>,
    profile_top_k: Option<u32>,
    profile_min_score: Option<f64>,
    binding_top_k: usize,
) -> KnowledgeRetrievalPlan {
    let methods = if request_methods.is_empty() {
        profile_strategy
            .map(retrieval_methods_for_strategy)
            .unwrap_or_else(default_rag_methods)
    } else {
        request_methods.to_vec()
    };

    KnowledgeRetrievalPlan {
        methods,
        top_k: profile_top_k.or(Some(binding_top_k as u32)),
        min_score: profile_min_score,
    }
}

pub fn kernel_methods_for_retrieval(
    methods: &[KnowledgeRetrievalMethod],
) -> Vec<sdkwork_agent_kernel::KnowledgeRetrievalMethod> {
    methods
        .iter()
        .map(|method| match method {
            KnowledgeRetrievalMethod::Keyword | KnowledgeRetrievalMethod::FullText => {
                sdkwork_agent_kernel::KnowledgeRetrievalMethod::Keyword
            }
            KnowledgeRetrievalMethod::Vector => {
                sdkwork_agent_kernel::KnowledgeRetrievalMethod::Vector
            }
            KnowledgeRetrievalMethod::Hybrid => {
                sdkwork_agent_kernel::KnowledgeRetrievalMethod::Hybrid
            }
            _ => sdkwork_agent_kernel::KnowledgeRetrievalMethod::Hybrid,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strategy_keyword_maps_to_keyword_method() {
        assert_eq!(
            retrieval_methods_for_strategy("keyword"),
            vec![KnowledgeRetrievalMethod::Keyword]
        );
    }

    #[test]
    fn strategy_hybrid_is_default_rag_plan() {
        assert_eq!(
            retrieval_methods_for_strategy("hybrid"),
            default_rag_methods()
        );
    }
}
