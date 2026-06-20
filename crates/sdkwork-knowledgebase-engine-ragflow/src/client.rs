//! RAGFlow retrieval HTTP client (adapter-local; handlers must not call RAGFlow directly).

use reqwest::Client;
use sdkwork_knowledgebase_contract::knowledge_engine::{
    KnowledgeEngineDocumentRef, KnowledgeEngineError, KnowledgeEngineSearchHit,
    KnowledgeEngineSearchResult,
};
use serde::Deserialize;

use crate::config::RagflowConnectorConfig;
use crate::RAGFLOW_IMPLEMENTATION_ID;

#[derive(Clone)]
pub struct RagflowApiClient {
    config: RagflowConnectorConfig,
    http: Client,
}

impl RagflowApiClient {
    pub fn new(config: RagflowConnectorConfig) -> Self {
        Self {
            config,
            http: Client::new(),
        }
    }

    pub async fn connector_health(&self, dataset_id: &str) -> Result<(), KnowledgeEngineError> {
        let url = format!(
            "{}/api/v1/datasets?id={dataset_id}",
            self.config.base_url.trim_end_matches('/')
        );
        let response = self
            .http
            .get(url)
            .header("Authorization", format!("Bearer {}", self.config.api_key))
            .send()
            .await
            .map_err(|error| KnowledgeEngineError::Internal(error.to_string()))?;

        if !response.status().is_success() {
            return Err(KnowledgeEngineError::Internal(format!(
                "ragflow connector health failed with status {}",
                response.status()
            )));
        }

        let payload: RagflowApiResponse<RagflowDatasetList> = response
            .json()
            .await
            .map_err(|error| KnowledgeEngineError::Internal(error.to_string()))?;

        if payload.code != 0 {
            return Err(KnowledgeEngineError::Internal(format!(
                "ragflow connector health failed with code {}: {}",
                payload.code,
                payload.message.unwrap_or_default()
            )));
        }

        Ok(())
    }

    pub async fn retrieve(
        &self,
        space_id: u64,
        dataset_id: &str,
        query: &str,
        top_k: u32,
    ) -> Result<KnowledgeEngineSearchResult, KnowledgeEngineError> {
        let url = format!(
            "{}/api/v1/retrieval",
            self.config.base_url.trim_end_matches('/')
        );
        let response = self
            .http
            .post(url)
            .header("Authorization", format!("Bearer {}", self.config.api_key))
            .json(&serde_json::json!({
                "question": query,
                "dataset_ids": [dataset_id],
                "page_size": top_k,
            }))
            .send()
            .await
            .map_err(|error| KnowledgeEngineError::Internal(error.to_string()))?;

        if !response.status().is_success() {
            return Err(KnowledgeEngineError::Internal(format!(
                "ragflow retrieve failed with status {}",
                response.status()
            )));
        }

        let payload: RagflowApiResponse<RagflowRetrievalData> = response
            .json()
            .await
            .map_err(|error| KnowledgeEngineError::Internal(error.to_string()))?;

        if payload.code != 0 {
            return Err(KnowledgeEngineError::Internal(format!(
                "ragflow retrieve failed with code {}: {}",
                payload.code,
                payload.message.unwrap_or_default()
            )));
        }

        let chunks = payload.data.map(|data| data.chunks).unwrap_or_default();

        let hits = chunks
            .into_iter()
            .map(|chunk| map_chunk_to_hit(space_id, chunk))
            .collect();

        Ok(KnowledgeEngineSearchResult {
            implementation_id: RAGFLOW_IMPLEMENTATION_ID.to_string(),
            hits,
        })
    }
}

fn map_chunk_to_hit(space_id: u64, chunk: RagflowChunk) -> KnowledgeEngineSearchHit {
    let chunk_id = chunk.id.clone().unwrap_or_else(|| "unknown".to_string());
    let local_document_id = match chunk.document_id.as_deref() {
        Some(document_id)
            if !document_id.is_empty() && chunk_id != "unknown" && document_id != chunk_id =>
        {
            format!("{document_id}#{chunk_id}")
        }
        _ => chunk_id.clone(),
    };
    let title = chunk
        .document_keyword
        .or(chunk.document_name)
        .unwrap_or_else(|| chunk_id.clone());

    KnowledgeEngineSearchHit {
        document: KnowledgeEngineDocumentRef {
            document_id: format!("{space_id}/{local_document_id}"),
            title,
            source_uri: chunk
                .document_id
                .map(|value| format!("ragflow://document/{value}/chunks/{chunk_id}")),
        },
        snippet: chunk.content.unwrap_or_default(),
        score: chunk.similarity,
    }
}

#[derive(Debug, Deserialize)]
struct RagflowApiResponse<T> {
    code: i32,
    message: Option<String>,
    data: Option<T>,
}

#[derive(Debug, Deserialize)]
struct RagflowRetrievalData {
    #[serde(default)]
    chunks: Vec<RagflowChunk>,
}

#[derive(Debug, Deserialize)]
struct RagflowDatasetList {
    #[serde(default)]
    _data: Vec<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
struct RagflowChunk {
    id: Option<String>,
    content: Option<String>,
    document_id: Option<String>,
    document_keyword: Option<String>,
    document_name: Option<String>,
    similarity: Option<f64>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_ragflow_retrieve_payload_to_scoped_hits() {
        let payload = r#"{
            "code": 0,
            "data": {
                "chunks": [
                    {
                        "id": "chunk-1",
                        "content": "ragflow snippet",
                        "document_id": "doc-9",
                        "document_keyword": "Policy Doc",
                        "similarity": 0.87
                    }
                ]
            }
        }"#;
        let parsed: RagflowApiResponse<RagflowRetrievalData> =
            serde_json::from_str(payload).expect("parse");
        let chunk = parsed
            .data
            .expect("data")
            .chunks
            .into_iter()
            .next()
            .expect("chunk");
        let hit = map_chunk_to_hit(11, chunk);

        assert_eq!(hit.document.document_id, "11/doc-9#chunk-1");
        assert_eq!(hit.document.title, "Policy Doc");
        assert_eq!(hit.snippet, "ragflow snippet");
        assert_eq!(hit.score, Some(0.87));
    }
}
