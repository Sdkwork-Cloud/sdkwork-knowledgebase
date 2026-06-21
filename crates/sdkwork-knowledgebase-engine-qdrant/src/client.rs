//! Qdrant REST HTTP client (adapter-local).

use reqwest::{Client, RequestBuilder};
use sdkwork_knowledgebase_contract::knowledge_engine::{
    KnowledgeEngineDocument, KnowledgeEngineDocumentRef, KnowledgeEngineError,
    KnowledgeEngineSearchHit, KnowledgeEngineSearchResult,
};
use serde::Deserialize;
use serde_json::Value;

use crate::config::{QdrantConnectorConfig, QDRANT_QUERY_MODEL_ENV};
use crate::QDRANT_IMPLEMENTATION_ID;

#[derive(Clone)]
pub struct QdrantApiClient {
    config: QdrantConnectorConfig,
    http: Client,
}

impl QdrantApiClient {
    pub fn new(config: QdrantConnectorConfig) -> Self {
        Self {
            config,
            http: Client::new(),
        }
    }

    fn authed(&self, builder: RequestBuilder) -> RequestBuilder {
        match self.config.api_key.as_deref() {
            Some(api_key) => builder.header("api-key", api_key),
            None => builder,
        }
    }

    fn collection_url(&self, collection_name: &str, suffix: &str) -> String {
        format!(
            "{}/collections/{collection_name}{suffix}",
            self.config.base_url.trim_end_matches('/'),
        )
    }

    pub async fn connector_health(
        &self,
        collection_name: &str,
    ) -> Result<(), KnowledgeEngineError> {
        let url = self.collection_url(collection_name, "");
        let response = self
            .authed(self.http.get(url))
            .send()
            .await
            .map_err(|error| KnowledgeEngineError::Internal(error.to_string()))?;

        if response.status().is_success() {
            Ok(())
        } else {
            Err(KnowledgeEngineError::Internal(format!(
                "qdrant connector health failed with status {}",
                response.status()
            )))
        }
    }

    pub async fn query_points(
        &self,
        space_id: u64,
        collection_name: &str,
        query: &str,
        top_k: u32,
    ) -> Result<KnowledgeEngineSearchResult, KnowledgeEngineError> {
        let query_model = self.config.query_model.as_deref().ok_or_else(|| {
            KnowledgeEngineError::Validation(format!(
                "Qdrant text query requires {QDRANT_QUERY_MODEL_ENV} for server-side embedding"
            ))
        })?;

        let mut body = serde_json::json!({
            "query": {
                "text": query,
                "model": query_model,
            },
            "limit": top_k,
            "with_payload": true,
        });
        if let Some(using_vector) = self.config.using_vector.as_deref() {
            body["using"] = Value::String(using_vector.to_string());
        }

        let url = self.collection_url(collection_name, "/points/query");
        let response = self
            .authed(self.http.post(url))
            .json(&body)
            .send()
            .await
            .map_err(|error| KnowledgeEngineError::Internal(error.to_string()))?;

        if !response.status().is_success() {
            return Err(KnowledgeEngineError::Internal(format!(
                "qdrant points query failed with status {}",
                response.status()
            )));
        }

        let payload: QdrantQueryResponse = response
            .json()
            .await
            .map_err(|error| KnowledgeEngineError::Internal(error.to_string()))?;

        let hits = payload
            .result
            .map(|result| map_points_to_hits(space_id, result.points))
            .unwrap_or_default();

        Ok(KnowledgeEngineSearchResult {
            implementation_id: QDRANT_IMPLEMENTATION_ID.to_string(),
            hits,
        })
    }

    pub async fn get_point(
        &self,
        collection_name: &str,
        point_id: &str,
    ) -> Result<KnowledgeEngineDocument, KnowledgeEngineError> {
        let parsed_id = parse_point_id(point_id);
        let url = self.collection_url(collection_name, "/points");
        let response = self
            .authed(self.http.post(url))
            .json(&serde_json::json!({
                "ids": [parsed_id],
                "with_payload": true,
            }))
            .send()
            .await
            .map_err(|error| KnowledgeEngineError::Internal(error.to_string()))?;

        if !response.status().is_success() {
            return Err(KnowledgeEngineError::Internal(format!(
                "qdrant get point failed with status {}",
                response.status()
            )));
        }

        let payload: QdrantGetPointsResponse = response
            .json()
            .await
            .map_err(|error| KnowledgeEngineError::Internal(error.to_string()))?;

        let point = payload.result.into_iter().next().ok_or_else(|| {
            KnowledgeEngineError::NotFound(format!(
                "qdrant point not found: collection={collection_name} id={point_id}"
            ))
        })?;

        let payload_value = point.payload.unwrap_or(Value::Null);
        let title = payload_title(&payload_value).unwrap_or_else(|| point_id.to_string());
        let content = payload_content(&payload_value);
        let point_id_string = point_id_string(&point.id);

        Ok(KnowledgeEngineDocument {
            document_id: format!("{title}#{point_id_string}"),
            title,
            content,
            source_uri: payload_value
                .get("source")
                .or_else(|| payload_value.get("url"))
                .and_then(Value::as_str)
                .filter(|value| !value.is_empty())
                .map(str::to_string),
        })
    }

    pub async fn get_collection(
        &self,
        collection_name: &str,
    ) -> Result<QdrantCollectionInfo, KnowledgeEngineError> {
        let url = self.collection_url(collection_name, "");
        let response = self
            .authed(self.http.get(url))
            .send()
            .await
            .map_err(|error| KnowledgeEngineError::Internal(error.to_string()))?;

        if !response.status().is_success() {
            return Err(KnowledgeEngineError::Internal(format!(
                "qdrant get collection failed with status {}",
                response.status()
            )));
        }

        let payload: QdrantCollectionResponse = response
            .json()
            .await
            .map_err(|error| KnowledgeEngineError::Internal(error.to_string()))?;

        payload.result.ok_or_else(|| {
            KnowledgeEngineError::Internal("qdrant collection payload missing result".to_string())
        })
    }
}

fn map_points_to_hits(
    space_id: u64,
    points: Vec<QdrantScoredPoint>,
) -> Vec<KnowledgeEngineSearchHit> {
    points
        .into_iter()
        .map(|point| {
            let payload = point.payload.unwrap_or(Value::Null);
            let point_id = point_id_string(&point.id);
            let title = payload_title(&payload).unwrap_or_else(|| point_id.clone());
            let local_document_id = format!("{title}#{point_id}");
            let snippet = payload_content(&payload);

            KnowledgeEngineSearchHit {
                document: KnowledgeEngineDocumentRef {
                    document_id: format!("{space_id}/{local_document_id}"),
                    title: title.clone(),
                    source_uri: payload
                        .get("source")
                        .or_else(|| payload.get("url"))
                        .and_then(Value::as_str)
                        .filter(|value| !value.is_empty())
                        .map(str::to_string),
                },
                snippet,
                score: point.score,
            }
        })
        .collect()
}

fn payload_title(payload: &Value) -> Option<String> {
    payload
        .get("title")
        .or_else(|| payload.get("source"))
        .or_else(|| payload.get("name"))
        .or_else(|| payload.get("document"))
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
}

fn payload_content(payload: &Value) -> String {
    payload
        .get("text")
        .or_else(|| payload.get("content"))
        .or_else(|| payload.get("page_content"))
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string()
}

fn point_id_string(id: &Value) -> String {
    match id {
        Value::String(value) => value.clone(),
        Value::Number(value) => value.to_string(),
        other => other.to_string(),
    }
}

fn parse_point_id(point_id: &str) -> Value {
    if let Ok(value) = point_id.parse::<u64>() {
        Value::Number(value.into())
    } else {
        Value::String(point_id.to_string())
    }
}

#[derive(Debug, Deserialize)]
struct QdrantQueryResponse {
    result: Option<QdrantQueryResult>,
}

#[derive(Debug, Deserialize)]
struct QdrantQueryResult {
    #[serde(default)]
    points: Vec<QdrantScoredPoint>,
}

#[derive(Debug, Deserialize)]
struct QdrantScoredPoint {
    id: Value,
    score: Option<f64>,
    payload: Option<Value>,
}

#[derive(Debug, Deserialize)]
struct QdrantGetPointsResponse {
    #[serde(default)]
    result: Vec<QdrantPointRecord>,
}

#[derive(Debug, Deserialize)]
struct QdrantPointRecord {
    id: Value,
    payload: Option<Value>,
}

#[derive(Debug, Deserialize)]
struct QdrantCollectionResponse {
    result: Option<QdrantCollectionInfo>,
}

#[derive(Debug, Deserialize)]
pub struct QdrantCollectionInfo {
    #[serde(default)]
    pub points_count: Option<u64>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_qdrant_query_payload_to_scoped_hits() {
        let points = vec![QdrantScoredPoint {
            id: Value::String("pt-1".to_string()),
            score: Some(0.91),
            payload: Some(serde_json::json!({
                "title": "Policy Doc",
                "text": "policy snippet",
                "source": "file://policy.txt"
            })),
        }];
        let hits = map_points_to_hits(4, points);

        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].document.document_id, "4/Policy Doc#pt-1");
        assert_eq!(hits[0].snippet, "policy snippet");
        assert_eq!(hits[0].score, Some(0.91));
    }
}
