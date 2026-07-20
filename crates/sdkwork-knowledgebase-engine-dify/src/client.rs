//! Dify dataset retrieve HTTP client (adapter-local; handlers must not call Dify directly).

use reqwest::Method;
use sdkwork_knowledgebase_contract::knowledge_engine::{
    KnowledgeEngineDocument, KnowledgeEngineDocumentRef, KnowledgeEngineError,
    KnowledgeEngineSearchHit, KnowledgeEngineSearchResult,
};
use sdkwork_knowledgebase_provider_runtime::{
    ProviderExecutionContext, ProviderHttpRequest, ProviderOperation, ProviderRuntime,
};
use serde::Deserialize;

use crate::config::DifyConnectorConfig;
use crate::DIFY_IMPLEMENTATION_ID;

#[derive(Clone)]
pub struct DifyApiClient {
    config: DifyConnectorConfig,
    http: ProviderRuntime,
}

impl DifyApiClient {
    pub fn new(config: DifyConnectorConfig) -> Self {
        let http = ProviderRuntime::for_base_url(&config.base_url)
            .expect("Dify base URL must satisfy Provider Runtime target policy");
        Self { config, http }
    }

    fn health_context(&self) -> ProviderExecutionContext {
        ProviderExecutionContext::for_system_health(DIFY_IMPLEMENTATION_ID)
    }

    pub async fn connector_health(&self, dataset_id: &str) -> Result<(), KnowledgeEngineError> {
        let url = format!(
            "{}/datasets/{dataset_id}",
            self.config.base_url.trim_end_matches('/')
        );
        let request = ProviderHttpRequest::new(ProviderOperation::Health, Method::GET, url)
            .map_err(KnowledgeEngineError::from)?
            .bearer_auth(self.config.api_key.as_str())
            .map_err(KnowledgeEngineError::from)?
            .idempotent(true);
        self.http
            .execute(&self.health_context(), request)
            .await
            .map_err(KnowledgeEngineError::from)?;
        Ok(())
    }

    pub async fn retrieve(
        &self,
        context: &ProviderExecutionContext,
        space_id: u64,
        dataset_id: &str,
        query: &str,
        top_k: u32,
    ) -> Result<KnowledgeEngineSearchResult, KnowledgeEngineError> {
        let url = format!(
            "{}/datasets/{dataset_id}/retrieve",
            self.config.base_url.trim_end_matches('/')
        );
        let request = ProviderHttpRequest::new(ProviderOperation::Search, Method::POST, url)
            .map_err(KnowledgeEngineError::from)?
            .bearer_auth(self.config.api_key.as_str())
            .map_err(KnowledgeEngineError::from)?
            .json(&serde_json::json!({
                "query": query,
                "top_k": top_k,
            }))
            .map_err(KnowledgeEngineError::from)?
            .idempotent(true);
        let response = self
            .http
            .execute(context, request)
            .await
            .map_err(KnowledgeEngineError::from)?;
        let payload: DifyRetrieveResponse = response.json().map_err(KnowledgeEngineError::from)?;

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
        context: &ProviderExecutionContext,
        dataset_id: &str,
        document_id: &str,
        segment_id: &str,
    ) -> Result<KnowledgeEngineDocument, KnowledgeEngineError> {
        let url = format!(
            "{}/datasets/{dataset_id}/documents/{document_id}/segments/{segment_id}",
            self.config.base_url.trim_end_matches('/')
        );
        let request = ProviderHttpRequest::new(ProviderOperation::Read, Method::GET, url)
            .map_err(KnowledgeEngineError::from)?
            .bearer_auth(self.config.api_key.as_str())
            .map_err(KnowledgeEngineError::from)?
            .idempotent(true);
        let response = self
            .http
            .execute(context, request)
            .await
            .map_err(KnowledgeEngineError::from)?;
        let payload: DifySegmentDetailResponse =
            response.json().map_err(KnowledgeEngineError::from)?;

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
