//! Flowise document-store vector query HTTP client (adapter-local).

use reqwest::Client;
use sdkwork_knowledgebase_contract::knowledge_engine::{
    KnowledgeEngineDocument, KnowledgeEngineDocumentRef, KnowledgeEngineError,
    KnowledgeEngineSearchHit, KnowledgeEngineSearchResult,
};
use serde::Deserialize;
use serde_json::Value;
use sha2::{Digest, Sha256};

use crate::config::FlowiseConnectorConfig;
use crate::FLOWISE_IMPLEMENTATION_ID;

#[derive(Clone)]
pub struct FlowiseApiClient {
    config: FlowiseConnectorConfig,
    http: Client,
}

impl FlowiseApiClient {
    pub fn new(config: FlowiseConnectorConfig) -> Self {
        Self {
            config,
            http: Client::new(),
        }
    }

    pub async fn connector_health(&self, store_id: &str) -> Result<(), KnowledgeEngineError> {
        let url = format!(
            "{}/api/v1/document-store/store/{store_id}",
            self.config.base_url.trim_end_matches('/')
        );
        let response = self
            .http
            .get(url)
            .bearer_auth(&self.config.api_key)
            .send()
            .await
            .map_err(|error| KnowledgeEngineError::Internal(error.to_string()))?;

        if response.status().is_success() {
            Ok(())
        } else {
            Err(KnowledgeEngineError::Internal(format!(
                "flowise connector health failed with status {}",
                response.status()
            )))
        }
    }

    pub async fn query_vector_store(
        &self,
        space_id: u64,
        store_id: &str,
        query: &str,
        top_k: u32,
    ) -> Result<KnowledgeEngineSearchResult, KnowledgeEngineError> {
        let url = format!(
            "{}/api/v1/document-store/vectorstore/query",
            self.config.base_url.trim_end_matches('/')
        );
        let response = self
            .http
            .post(url)
            .bearer_auth(&self.config.api_key)
            .json(&serde_json::json!({
                "storeId": store_id,
                "query": query,
            }))
            .send()
            .await
            .map_err(|error| KnowledgeEngineError::Internal(error.to_string()))?;

        if !response.status().is_success() {
            return Err(KnowledgeEngineError::Internal(format!(
                "flowise vectorstore query failed with status {}",
                response.status()
            )));
        }

        let payload: FlowiseVectorQueryResponse = response
            .json()
            .await
            .map_err(|error| KnowledgeEngineError::Internal(error.to_string()))?;

        let hits = payload
            .docs
            .into_iter()
            .take(top_k as usize)
            .enumerate()
            .map(|(index, doc)| map_doc_to_hit(space_id, doc, index))
            .collect();

        Ok(KnowledgeEngineSearchResult {
            implementation_id: FLOWISE_IMPLEMENTATION_ID.to_string(),
            hits,
        })
    }

    pub async fn read_chunk(
        &self,
        space_id: u64,
        store_id: &str,
        document_hint: &str,
        chunk_id: &str,
    ) -> Result<KnowledgeEngineDocument, KnowledgeEngineError> {
        let search = self
            .query_vector_store(space_id, store_id, document_hint, 25)
            .await?;

        let hit = search
            .hits
            .into_iter()
            .find(|candidate| {
                let local = candidate
                    .document
                    .document_id
                    .split_once('/')
                    .map(|(_, rest)| rest)
                    .unwrap_or(candidate.document.document_id.as_str());
                local
                    .split_once('#')
                    .is_some_and(|(title, id)| title == document_hint && id == chunk_id)
            })
            .ok_or_else(|| {
                KnowledgeEngineError::NotFound(format!(
                    "flowise chunk not found in store_id={store_id} chunk_id={chunk_id}"
                ))
            })?;

        let local_document_id = hit
            .document
            .document_id
            .split_once('/')
            .map(|(_, rest)| rest.to_string())
            .unwrap_or_else(|| format!("{document_hint}#{chunk_id}"));

        Ok(KnowledgeEngineDocument {
            document_id: local_document_id,
            title: hit.document.title,
            content: hit.snippet,
            source_uri: hit.document.source_uri,
        })
    }
}

fn map_doc_to_hit(space_id: u64, doc: FlowiseDocument, rank: usize) -> KnowledgeEngineSearchHit {
    let content = doc.page_content.unwrap_or_default();
    let title = doc
        .metadata
        .as_ref()
        .and_then(metadata_title)
        .unwrap_or_else(|| "document".to_string());
    let chunk_id = chunk_id_from_content(&content);
    let local_document_id = format!("{title}#{chunk_id}");
    let source_uri = doc.metadata.as_ref().and_then(|metadata| {
        metadata
            .get("source")
            .or_else(|| metadata.get("url"))
            .and_then(Value::as_str)
            .filter(|value| !value.is_empty())
            .map(str::to_string)
    });

    let score = doc
        .metadata
        .as_ref()
        .and_then(|metadata| metadata.get("score"))
        .and_then(Value::as_f64)
        .or_else(|| Some(1.0 / (rank as f64 + 1.0)));

    KnowledgeEngineSearchHit {
        document: KnowledgeEngineDocumentRef {
            document_id: format!("{space_id}/{local_document_id}"),
            title: title.clone(),
            source_uri,
        },
        snippet: content,
        score,
    }
}

fn metadata_title(metadata: &Value) -> Option<String> {
    metadata
        .get("source")
        .or_else(|| metadata.get("title"))
        .or_else(|| metadata.get("name"))
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
}

pub fn chunk_id_from_content(content: &str) -> String {
    let digest = Sha256::digest(content.as_bytes());
    format!("{:x}", digest)[..16].to_string()
}

#[derive(Debug, Deserialize)]
struct FlowiseVectorQueryResponse {
    #[serde(default)]
    docs: Vec<FlowiseDocument>,
}

#[derive(Debug, Deserialize)]
struct FlowiseDocument {
    #[serde(rename = "pageContent")]
    page_content: Option<String>,
    #[serde(default)]
    metadata: Option<Value>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_flowise_vector_query_payload_to_scoped_hits() {
        let doc = FlowiseDocument {
            page_content: Some("policy snippet".to_string()),
            metadata: Some(serde_json::json!({
                "source": "Policy Doc",
                "url": "file://policy.txt"
            })),
        };
        let hit = map_doc_to_hit(4, doc, 0);
        let chunk_id = chunk_id_from_content("policy snippet");

        assert_eq!(hit.document.document_id, format!("4/Policy Doc#{chunk_id}"));
        assert_eq!(hit.document.title, "Policy Doc");
        assert_eq!(hit.snippet, "policy snippet");
        assert!(hit.score.is_some());
    }
}
