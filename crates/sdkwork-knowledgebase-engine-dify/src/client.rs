//! Dify dataset retrieve HTTP client (adapter-local; handlers must not call Dify directly).

use reqwest::Client;
use sdkwork_knowledgebase_contract::knowledge_engine::{
    KnowledgeEngineDocument, KnowledgeEngineDocumentRef, KnowledgeEngineError,
    KnowledgeEngineSearchHit, KnowledgeEngineSearchResult,
};
use serde::Deserialize;

use crate::config::DifyConnectorConfig;
use crate::DIFY_IMPLEMENTATION_ID;

#[derive(Clone)]
pub struct DifyApiClient {
    config: DifyConnectorConfig,
    http: Client,
}

impl DifyApiClient {
    pub fn new(config: DifyConnectorConfig) -> Self {
        Self {
            config,
            http: Client::new(),
        }
    }

    pub async fn connector_health(&self, dataset_id: &str) -> Result<(), KnowledgeEngineError> {
        let url = format!(
            "{}/datasets/{dataset_id}",
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
                "dify connector health failed with status {}",
                response.status()
            )))
        }
    }

    pub async fn retrieve(
        &self,
        space_id: u64,
        dataset_id: &str,
        query: &str,
        top_k: u32,
    ) -> Result<KnowledgeEngineSearchResult, KnowledgeEngineError> {
        let url = format!(
            "{}/datasets/{dataset_id}/retrieve",
            self.config.base_url.trim_end_matches('/')
        );
        let response = self
            .http
            .post(url)
            .bearer_auth(&self.config.api_key)
            .json(&serde_json::json!({
                "query": query,
                "top_k": top_k,
            }))
            .send()
            .await
            .map_err(|error| KnowledgeEngineError::Internal(error.to_string()))?;

        if !response.status().is_success() {
            return Err(KnowledgeEngineError::Internal(format!(
                "dify retrieve failed with status {}",
                response.status()
            )));
        }

        let payload: DifyRetrieveResponse = response
            .json()
            .await
            .map_err(|error| KnowledgeEngineError::Internal(error.to_string()))?;

        let hits = payload
            .records
            .into_iter()
            .map(|record| map_record_to_hit(space_id, record))
            .collect();

        Ok(KnowledgeEngineSearchResult {
            implementation_id: DIFY_IMPLEMENTATION_ID.to_string(),
            hits,
        })
    }

    pub async fn read_segment(
        &self,
        dataset_id: &str,
        document_id: &str,
        segment_id: &str,
    ) -> Result<KnowledgeEngineDocument, KnowledgeEngineError> {
        let url = format!(
            "{}/datasets/{dataset_id}/documents/{document_id}/segments/{segment_id}",
            self.config.base_url.trim_end_matches('/')
        );
        let response = self
            .http
            .get(url)
            .bearer_auth(&self.config.api_key)
            .send()
            .await
            .map_err(|error| KnowledgeEngineError::Internal(error.to_string()))?;

        if response.status() == reqwest::StatusCode::NOT_FOUND {
            return Err(KnowledgeEngineError::NotFound(format!(
                "dify segment not found: dataset={dataset_id} document={document_id} segment={segment_id}"
            )));
        }

        if !response.status().is_success() {
            return Err(KnowledgeEngineError::Internal(format!(
                "dify read segment failed with status {}",
                response.status()
            )));
        }

        let payload: DifySegmentDetailResponse = response
            .json()
            .await
            .map_err(|error| KnowledgeEngineError::Internal(error.to_string()))?;

        let segment = payload.data.ok_or_else(|| {
            KnowledgeEngineError::NotFound(format!(
                "dify segment payload missing for segment_id={segment_id}"
            ))
        })?;

        let title = segment
            .document
            .and_then(|document| document.name)
            .unwrap_or_else(|| document_id.to_string());

        Ok(KnowledgeEngineDocument {
            document_id: format!("{document_id}#{segment_id}"),
            title,
            content: segment.content.unwrap_or_default(),
            source_uri: Some(format!(
                "dify://documents/{document_id}/segments/{segment_id}"
            )),
        })
    }
}

fn map_record_to_hit(space_id: u64, record: DifyRetrieveRecord) -> KnowledgeEngineSearchHit {
    let segment = record.segment;
    let segment_id = segment
        .id
        .clone()
        .or_else(|| segment.document_id.clone())
        .unwrap_or_else(|| "unknown".to_string());
    let parent_document_id = segment.document_id.clone();
    let local_document_id = match parent_document_id.as_deref() {
        Some(document_id)
            if !document_id.is_empty() && segment_id != "unknown" && document_id != segment_id =>
        {
            format!("{document_id}#{segment_id}")
        }
        _ => segment_id.clone(),
    };
    let title = segment
        .document
        .and_then(|document| document.name)
        .unwrap_or_else(|| segment_id.clone());

    KnowledgeEngineSearchHit {
        document: KnowledgeEngineDocumentRef {
            document_id: format!("{space_id}/{local_document_id}"),
            title,
            source_uri: parent_document_id
                .map(|document_id| format!("dify://documents/{document_id}/segments/{segment_id}")),
        },
        snippet: segment.content.unwrap_or_default(),
        score: record.score,
    }
}

#[derive(Debug, Deserialize)]
struct DifyRetrieveResponse {
    #[serde(default)]
    records: Vec<DifyRetrieveRecord>,
}

#[derive(Debug, Deserialize)]
struct DifyRetrieveRecord {
    segment: DifySegment,
    score: Option<f64>,
}

#[derive(Debug, Deserialize)]
struct DifySegment {
    id: Option<String>,
    #[serde(rename = "document_id")]
    document_id: Option<String>,
    content: Option<String>,
    document: Option<DifySegmentDocument>,
}

#[derive(Debug, Deserialize)]
struct DifySegmentDocument {
    name: Option<String>,
}

#[derive(Debug, Deserialize)]
struct DifySegmentDetailResponse {
    data: Option<DifySegmentDetail>,
}

#[derive(Debug, Deserialize)]
struct DifySegmentDetail {
    content: Option<String>,
    document: Option<DifySegmentDocument>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_dify_retrieve_payload_to_scoped_hits() {
        let payload = r#"{
            "records": [
                {
                    "segment": {
                        "id": "seg-1",
                        "document_id": "doc-1",
                        "content": "hello world",
                        "document": { "name": "Doc A" }
                    },
                    "score": 0.91
                }
            ]
        }"#;
        let parsed: DifyRetrieveResponse = serde_json::from_str(payload).expect("parse");
        let hit = map_record_to_hit(7, parsed.records.into_iter().next().expect("record"));

        assert_eq!(hit.document.document_id, "7/doc-1#seg-1");
        assert_eq!(hit.document.title, "Doc A");
        assert_eq!(hit.snippet, "hello world");
        assert_eq!(hit.score, Some(0.91));
    }
}
