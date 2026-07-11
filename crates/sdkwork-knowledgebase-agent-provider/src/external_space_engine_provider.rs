use sdkwork_agent_kernel::{
    KernelError, KernelErrorSource, KernelResult, KnowledgeDocument, KnowledgeDocumentFilter,
    KnowledgeDocumentKind, KnowledgeProvider, KnowledgeRetrievalMethod, KnowledgeSearchRequest,
    KnowledgeSearchResult, ProviderHealth, ProviderManifest, RedactionClassification, TrustLevel,
};
use sdkwork_knowledgebase_contract::knowledge_engine::KnowledgeEngineSearchHit;
use sdkwork_utils_rust::is_blank;
use std::sync::Arc;

use crate::async_bridge::block_on_async;
use crate::knowledge_access::SpaceKnowledgeEngineClient;
use crate::mapper::{
    format_scoped_document_id, parse_namespace_space_id, parse_scoped_document_id,
};

pub struct SpaceEngineKnowledgeProvider {
    provider_id: String,
    client: Arc<dyn SpaceKnowledgeEngineClient>,
    tenant_id: u64,
}

impl SpaceEngineKnowledgeProvider {
    pub fn new(
        provider_id: impl Into<String>,
        client: Arc<dyn SpaceKnowledgeEngineClient>,
        tenant_id: u64,
    ) -> Self {
        Self {
            provider_id: provider_id.into(),
            client,
            tenant_id,
        }
    }
}

impl KnowledgeProvider for SpaceEngineKnowledgeProvider {
    fn provider_manifest(&self) -> ProviderManifest {
        ProviderManifest::new(
            &self.provider_id,
            "knowledge",
            "space-engine-knowledge-provider",
            env!("CARGO_PKG_VERSION"),
            vec![
                "knowledge.search".to_string(),
                "knowledge.read".to_string(),
                "knowledge.list".to_string(),
            ],
        )
    }

    fn search(&self, request: KnowledgeSearchRequest) -> KernelResult<Vec<KnowledgeSearchResult>> {
        if is_blank(Some(request.query.as_str())) {
            return Err(KernelError::validation(
                "external knowledge search query must not be blank",
            ));
        }

        let space_id = parse_namespace_space_id(request.namespace.as_deref())?;
        let tenant_id =
            crate::mapper::parse_tenant_id(request.tenant_id.as_deref()).unwrap_or(self.tenant_id);
        let top_k = request.top_k.max(1) as u32;

        let client = Arc::clone(&self.client);
        let query = request.query.clone();

        let result = block_on_async(async move {
            client
                .search_space(tenant_id, space_id, &query, top_k)
                .await
        })
        .map_err(|error| {
            KernelError::provider_error("external_knowledge.search_failed", error.to_string())
                .with_provider(self.provider_id.clone())
                .from_source(KernelErrorSource::Provider)
        })?
        .map_err(|message| {
            KernelError::provider_error("external_knowledge.search_failed", message)
                .with_provider(self.provider_id.clone())
                .from_source(KernelErrorSource::Provider)
        })?;

        Ok(result
            .hits
            .iter()
            .map(|hit| engine_hit_to_search_result(space_id, hit))
            .collect())
    }

    fn read(&self, document_id: &str) -> KernelResult<KnowledgeDocument> {
        let (space_id, scoped_document_id) =
            parse_scoped_document_id(document_id).ok_or_else(|| {
                KernelError::validation(
                    "external knowledge read requires scoped document_id {space_id}/{document_id}",
                )
            })?;

        let client = Arc::clone(&self.client);
        let tenant_id = self.tenant_id;
        let document = block_on_async(async move {
            client
                .read_space_document(tenant_id, space_id, &scoped_document_id)
                .await
        })
        .map_err(|error| {
            KernelError::provider_error("external_knowledge.read_failed", error.to_string())
                .with_provider(self.provider_id.clone())
                .from_source(KernelErrorSource::Provider)
        })?
        .map_err(|message| {
            KernelError::provider_error("external_knowledge.read_failed", message)
                .with_provider(self.provider_id.clone())
                .from_source(KernelErrorSource::Provider)
        })?;

        Ok(KnowledgeDocument::new(
            format_scoped_document_id(space_id, &document.document_id),
            KnowledgeDocumentKind::ExternalReference,
            document.title,
            document.content,
        )
        .with_namespace(format!("space:{space_id}"))
        .with_source_uri(document.source_uri.unwrap_or_default()))
    }

    fn list(&self, _filter: KnowledgeDocumentFilter) -> KernelResult<Vec<KnowledgeDocument>> {
        Err(KernelError::provider_error(
            "external_knowledge.list_unsupported",
            "external knowledge list is not wired for this provider yet",
        )
        .with_provider(self.provider_id.clone())
        .from_source(KernelErrorSource::Provider))
    }

    fn health(&self) -> ProviderHealth {
        ProviderHealth::available()
    }
}

pub fn engine_hit_to_search_result(
    space_id: u64,
    hit: &KnowledgeEngineSearchHit,
) -> KnowledgeSearchResult {
    let document_id = if hit.document.document_id.contains('/') {
        hit.document.document_id.clone()
    } else {
        format!("{space_id}/{}", hit.document.document_id)
    };

    let mut result = KnowledgeSearchResult::new(
        document_id,
        KnowledgeDocumentKind::ExternalReference,
        hit.document.title.clone(),
        KnowledgeRetrievalMethod::External,
    )
    .with_snippet(hit.snippet.clone())
    .with_trust_level(TrustLevel::TrustedHost)
    .with_redaction_classification(RedactionClassification::TenantSensitive)
    .with_metadata("sdkwork.knowledge.space_id", space_id.to_string());

    if let Some(score) = hit.score {
        result = result.with_score(score);
    }
    if let Some(source_uri) = hit.document.source_uri.clone() {
        result = result.with_source_uri(source_uri);
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use sdkwork_knowledgebase_contract::knowledge_engine::KnowledgeEngineDocumentRef;

    #[test]
    fn engine_hit_preserves_scoped_document_ref() {
        let hit = KnowledgeEngineSearchHit {
            document: KnowledgeEngineDocumentRef {
                document_id: "42/seg-1".to_string(),
                title: "Doc".to_string(),
                source_uri: None,
            },
            snippet: "snippet".to_string(),
            score: Some(0.9),
        };

        let result = engine_hit_to_search_result(42, &hit);
        assert_eq!(result.document_id, "42/seg-1");
    }

    #[tokio::test(flavor = "current_thread")]
    async fn space_engine_provider_search_works_on_current_thread_runtime() {
        use async_trait::async_trait;
        use sdkwork_knowledgebase_contract::knowledge_engine::{
            KnowledgeEngineDocumentRef, KnowledgeEngineSearchHit, KnowledgeEngineSearchResult,
        };

        struct FakeClient;

        #[async_trait]
        impl SpaceKnowledgeEngineClient for FakeClient {
            async fn search_space(
                &self,
                _tenant_id: u64,
                space_id: u64,
                query: &str,
                top_k: u32,
            ) -> Result<KnowledgeEngineSearchResult, String> {
                assert_eq!(space_id, 7);
                assert_eq!(query, "hello");
                assert_eq!(top_k, 2);
                Ok(KnowledgeEngineSearchResult {
                    implementation_id: "engine.knowledge.external.dify".to_string(),
                    hits: vec![KnowledgeEngineSearchHit {
                        document: KnowledgeEngineDocumentRef {
                            document_id: "7/seg".to_string(),
                            title: "Doc".to_string(),
                            source_uri: None,
                        },
                        snippet: "answer".to_string(),
                        score: Some(0.5),
                    }],
                })
            }

            async fn agent_provider_id_for_space(&self, _space_id: u64) -> Result<String, String> {
                Ok("provider.knowledge.external.dify".to_string())
            }

            async fn read_space_document(
                &self,
                _tenant_id: u64,
                _space_id: u64,
                _scoped_document_id: &str,
            ) -> Result<
                sdkwork_knowledgebase_contract::knowledge_engine::KnowledgeEngineDocument,
                String,
            > {
                Err("unsupported in test fake".to_string())
            }
        }

        let provider = SpaceEngineKnowledgeProvider::new(
            "provider.knowledge.external.dify",
            Arc::new(FakeClient),
            1,
        );
        let mut search_request = sdkwork_agent_kernel::KnowledgeSearchRequest::new("hello");
        search_request.tenant_id = Some("1".to_string());
        search_request.namespace = Some("space:7".to_string());
        search_request.top_k = 2;
        let hits = provider.search(search_request).expect("search");

        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].document_id, "7/seg");
    }
}
