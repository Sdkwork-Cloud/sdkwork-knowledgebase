use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct KnowledgeRetrievalProfileRequest {
    pub name: String,

    pub strategy: String,

    #[serde(rename = "topK")]
    pub top_k: i64,

    #[serde(rename = "minScore")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub min_score: Option<f64>,

    #[serde(rename = "rerankEnabled")]
    pub rerank_enabled: bool,

    #[serde(rename = "contextBudgetTokens")]
    pub context_budget_tokens: i64,

    pub status: String,
}
