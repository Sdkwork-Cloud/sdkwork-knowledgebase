//! Onyx unified search HTTP client (adapter-local; handlers must not call Onyx directly).

use reqwest::Client;
use sdkwork_knowledgebase_contract::knowledge_engine::{
    KnowledgeEngineDocument, KnowledgeEngineDocumentRef, KnowledgeEngineError,
    KnowledgeEngineSearchHit, KnowledgeEngineSearchResult,
};
use serde::Deserialize;

use crate::config::OnyxConnectorConfig;
use crate::ONYX_IMPLEMENTATION_ID;

#[derive(Clone)]
pub struct OnyxApiClient {
    config: OnyxConnectorConfig,
    http: Client,
}

impl OnyxApiClient {
    pub fn new(config: OnyxConnectorConfig) -> Self {
        Self {
            config,
            http: Client::new(),
        }
    }

    pub async fn connector_health(&self) -> Result<(), KnowledgeEngineError> {
        let url = format!("{}/health", self.config.base_url.trim_end_matches('/'));
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
                "onyx connector health failed with status {}",
                response.status()
            )))
        }
    }

    pub async fn search(
        &self,
        space_id: u64,
        query: &str,
    ) -> Result<KnowledgeEngineSearchResult, KnowledgeEngineError> {
        let url = format!("{}/search", self.config.base_url.trim_end_matches('/'));
        let response = self
            .http
            .post(url)
            .bearer_auth(&self.config.api_key)
            .json(&serde_json::json!({
                "query": query,
                "skip_query_expansion": false,
            }))
            .send()
            .await
            .map_err(|error| KnowledgeEngineError::Internal(error.to_string()))?;

        if !response.status().is_success() {
            return Err(KnowledgeEngineError::Internal(format!(
                "onyx search failed with status {}",
                response.status()
            )));
        }

        let payload: OnyxSearchResponse = response
            .json()
            .await
            .map_err(|error| KnowledgeEngineError::Internal(error.to_string()))?;

        if let Some(error) = payload.error.filter(|value| !value.is_empty()) {
            return Err(KnowledgeEngineError::Internal(format!(
                "onyx search failed: {error}"
            )));
        }

        let hits = payload
            .results
            .into_iter()
            .map(|result| map_result_to_hit(space_id, result))
            .collect();

        Ok(KnowledgeEngineSearchResult {
            implementation_id: ONYX_IMPLEMENTATION_ID.to_string(),
            hits,
        })
    }

    pub async fn read_url_document(
        &self,
        url: &str,
    ) -> Result<KnowledgeEngineDocument, KnowledgeEngineError> {
        let endpoint = format!("{}/open_urls", self.config.base_url.trim_end_matches('/'));
        let response = self
            .http
            .post(endpoint)
            .bearer_auth(&self.config.api_key)
            .json(&serde_json::json!({
                "urls": [url],
            }))
            .send()
            .await
            .map_err(|error| KnowledgeEngineError::Internal(error.to_string()))?;

        if !response.status().is_success() {
            return Err(KnowledgeEngineError::Internal(format!(
                "onyx open_urls failed with status {}",
                response.status()
            )));
        }

        let payload: OnyxOpenUrlsResponse = response
            .json()
            .await
            .map_err(|error| KnowledgeEngineError::Internal(error.to_string()))?;

        if let Some(error) = payload.error.filter(|value| !value.is_empty()) {
            return Err(KnowledgeEngineError::Internal(format!(
                "onyx open_urls failed: {error}"
            )));
        }

        let Some(result) = payload.results.into_iter().next() else {
            return Err(KnowledgeEngineError::NotFound(format!(
                "onyx document not found for url={url}"
            )));
        };

        let title = result.title.unwrap_or_else(|| url.to_string());
        let content = result.content.unwrap_or_default();

        Ok(KnowledgeEngineDocument {
            document_id: encode_url_document_id(url),
            title,
            content,
            source_uri: Some(url.to_string()),
        })
    }
}

pub fn encode_url_document_id(url: &str) -> String {
    format!("url:{url}")
}

pub fn decode_url_document_id(document_id: &str) -> Option<String> {
    document_id.strip_prefix("url:").map(str::to_string)
}

fn map_result_to_hit(space_id: u64, result: OnyxSearchResult) -> KnowledgeEngineSearchHit {
    let url = result.url.unwrap_or_default();
    let title = result.title.unwrap_or_else(|| url.clone());
    let local_document_id = if url.is_empty() {
        "unknown".to_string()
    } else {
        encode_url_document_id(&url)
    };

    KnowledgeEngineSearchHit {
        document: KnowledgeEngineDocumentRef {
            document_id: format!("{space_id}/{local_document_id}"),
            title,
            source_uri: if url.is_empty() { None } else { Some(url) },
        },
        snippet: result.content.unwrap_or_default(),
        score: None,
    }
}

#[derive(Debug, Deserialize)]
struct OnyxSearchResponse {
    #[serde(default)]
    results: Vec<OnyxSearchResult>,
    error: Option<String>,
}

#[derive(Debug, Deserialize)]
struct OnyxSearchResult {
    title: Option<String>,
    url: Option<String>,
    content: Option<String>,
    #[serde(rename = "source_type")]
    _source_type: Option<String>,
}

#[derive(Debug, Deserialize)]
struct OnyxOpenUrlsResponse {
    #[serde(default)]
    results: Vec<OnyxOpenUrlResult>,
    error: Option<String>,
}

#[derive(Debug, Deserialize)]
struct OnyxOpenUrlResult {
    title: Option<String>,
    content: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_onyx_search_payload_to_scoped_hits() {
        let payload = r#"{
            "results": [
                {
                    "title": "Reset Okta",
                    "url": "https://example.com/okta",
                    "content": "reset steps",
                    "source_type": "web"
                }
            ]
        }"#;
        let parsed: OnyxSearchResponse = serde_json::from_str(payload).expect("parse");
        let hit = map_result_to_hit(5, parsed.results.into_iter().next().expect("result"));

        assert_eq!(hit.document.document_id, "5/url:https://example.com/okta");
        assert_eq!(hit.document.title, "Reset Okta");
        assert_eq!(hit.snippet, "reset steps");
    }

    #[test]
    fn url_document_id_round_trips() {
        let url = "https://example.com/doc";
        let encoded = encode_url_document_id(url);
        assert_eq!(decode_url_document_id(&encoded).as_deref(), Some(url));
    }
}
