use async_trait::async_trait;
#[path = "support/okf_pagination.rs"]
mod okf_pagination_support;

use okf_pagination_support::validated_okf_test_page_size;
use sdkwork_intelligence_knowledgebase_service::knowledge_engine::{
    build_default_registry, KnowledgeEngineRuntimeDeps, KnowledgeEngineSpaceResolver,
    OkfNativeKnowledgeEngineDeps,
};
use sdkwork_intelligence_knowledgebase_service::ports::knowledge_document_store::{
    CreateKnowledgeDocumentRecord, KnowledgeDocumentStore, KnowledgeDocumentStoreError,
};
use sdkwork_intelligence_knowledgebase_service::ports::knowledge_drive_storage::{
    HeadKnowledgeObjectRequest, KnowledgeDriveStorage, KnowledgeObjectRef, KnowledgeStorageError,
    PutKnowledgeObjectRequest,
};
use sdkwork_intelligence_knowledgebase_service::ports::knowledge_okf_concept_store::{
    AppendKnowledgeOkfLogEntryRecord, CreateKnowledgeOkfConceptRevisionRecord,
    KnowledgeOkfConceptProjection, KnowledgeOkfConceptStore, KnowledgeOkfConceptStoreError,
    MarkKnowledgeOkfConceptCurrentRevisionRecord, UpsertKnowledgeOkfConceptRecord,
};
use sdkwork_intelligence_knowledgebase_service::ports::knowledge_retrieval_backend::{
    KnowledgeChunkSearchHit, KnowledgeChunkSearchRequest, KnowledgeRetrievalBackend,
    KnowledgeRetrievalBackendError,
};
use sdkwork_intelligence_knowledgebase_service::ports::knowledge_retrieval_trace_store::{
    CreateKnowledgeRetrievalHitRecord, CreateKnowledgeRetrievalTraceRecord,
    KnowledgeRetrievalTraceHitRecord, KnowledgeRetrievalTraceRecord, KnowledgeRetrievalTraceStore,
    KnowledgeRetrievalTraceStoreError,
};
use sdkwork_intelligence_knowledgebase_service::ports::knowledge_source_store::{
    CreateKnowledgeSourceRecord, KnowledgeSourceStore, KnowledgeSourceStoreError,
};
use sdkwork_intelligence_knowledgebase_service::ports::knowledge_space_store::{
    CreateKnowledgeSpaceRecord, KnowledgeSpaceStore, KnowledgeSpaceStoreError,
};
use sdkwork_knowledgebase_contract::knowledge_engine::KnowledgeEngineId;
use sdkwork_knowledgebase_contract::okf::{
    KnowledgeOkfConcept, KnowledgeOkfConceptRevision, OkfConceptSummary, OkfLogEntry,
};
use sdkwork_knowledgebase_contract::rag::KnowledgeAgentKnowledgeMode;
use sdkwork_knowledgebase_contract::source::{KnowledgeSource, KnowledgeSourceType};
use sdkwork_knowledgebase_contract::space::KnowledgeSpace;
use sdkwork_knowledgebase_engine_dify::{DifyConnectorConfig, DifyKnowledgeEngine};
use sdkwork_knowledgebase_engine_ragflow::{RagflowConnectorConfig, RagflowKnowledgeEngine};
use std::collections::HashMap;
use std::sync::Arc;

struct MockOkfConceptStore;

#[async_trait]
impl KnowledgeOkfConceptStore for MockOkfConceptStore {
    async fn upsert_concept(
        &self,
        _record: UpsertKnowledgeOkfConceptRecord,
    ) -> Result<KnowledgeOkfConcept, KnowledgeOkfConceptStoreError> {
        Err(KnowledgeOkfConceptStoreError::Internal(
            "unsupported in test fake".to_string(),
        ))
    }

    async fn create_revision(
        &self,
        _record: CreateKnowledgeOkfConceptRevisionRecord,
    ) -> Result<KnowledgeOkfConceptRevision, KnowledgeOkfConceptStoreError> {
        Err(KnowledgeOkfConceptStoreError::Internal(
            "unsupported in test fake".to_string(),
        ))
    }

    async fn next_revision_no(
        &self,
        _concept_row_id: u64,
    ) -> Result<u64, KnowledgeOkfConceptStoreError> {
        Err(KnowledgeOkfConceptStoreError::Internal(
            "unsupported in test fake".to_string(),
        ))
    }

    async fn mark_current_revision(
        &self,
        _record: MarkKnowledgeOkfConceptCurrentRevisionRecord,
    ) -> Result<KnowledgeOkfConcept, KnowledgeOkfConceptStoreError> {
        Err(KnowledgeOkfConceptStoreError::Internal(
            "unsupported in test fake".to_string(),
        ))
    }

    async fn list_concept_summaries(
        &self,
        _space_id: u64,
        _limit: Option<u32>,
    ) -> Result<Vec<OkfConceptSummary>, KnowledgeOkfConceptStoreError> {
        Ok(Vec::new())
    }

    async fn list_concept_summaries_page(
        &self,
        _space_id: u64,
        _cursor: Option<String>,
        page_size: u32,
    ) -> Result<(Vec<OkfConceptSummary>, Option<String>, bool), KnowledgeOkfConceptStoreError> {
        validated_okf_test_page_size(page_size)?;
        Ok((Vec::new(), None, false))
    }

    async fn list_concept_revisions_page(
        &self,
        _concept_row_id: u64,
        _cursor: Option<u64>,
        page_size: u32,
    ) -> Result<(Vec<KnowledgeOkfConceptRevision>, Option<u64>, bool), KnowledgeOkfConceptStoreError>
    {
        validated_okf_test_page_size(page_size)?;
        Ok((Vec::new(), None, false))
    }

    async fn append_log_entry(
        &self,
        _record: AppendKnowledgeOkfLogEntryRecord,
    ) -> Result<OkfLogEntry, KnowledgeOkfConceptStoreError> {
        Err(KnowledgeOkfConceptStoreError::Internal(
            "unsupported in test fake".to_string(),
        ))
    }

    async fn list_log_entries(
        &self,
        _space_id: u64,
    ) -> Result<Vec<OkfLogEntry>, KnowledgeOkfConceptStoreError> {
        Ok(Vec::new())
    }

    async fn batch_concept_projections_by_paths(
        &self,
        _space_id: u64,
        _logical_paths: Vec<String>,
    ) -> Result<Vec<KnowledgeOkfConceptProjection>, KnowledgeOkfConceptStoreError> {
        Ok(Vec::new())
    }

    async fn mark_concept_deleted(
        &self,
        _space_id: u64,
        _concept_row_id: u64,
    ) -> Result<KnowledgeOkfConcept, KnowledgeOkfConceptStoreError> {
        Err(KnowledgeOkfConceptStoreError::Internal(
            "unsupported in test fake".to_string(),
        ))
    }
}

struct MockDriveStorage;

struct MockDocumentStore;

#[async_trait]
impl KnowledgeDocumentStore for MockDocumentStore {
    async fn create_document(
        &self,
        _record: CreateKnowledgeDocumentRecord,
    ) -> Result<
        sdkwork_knowledgebase_contract::document::KnowledgeDocument,
        KnowledgeDocumentStoreError,
    > {
        Err(KnowledgeDocumentStoreError::Internal(
            "unsupported in test fake".to_string(),
        ))
    }
}

#[async_trait]
impl KnowledgeDriveStorage for MockDriveStorage {
    async fn put_object(
        &self,
        request: PutKnowledgeObjectRequest,
    ) -> Result<KnowledgeObjectRef, KnowledgeStorageError> {
        Ok(KnowledgeObjectRef {
            storage_provider_id: "mock".to_string(),
            bucket: "mock".to_string(),
            object_key: request.logical_path.clone(),
            logical_path: request.logical_path,
            object_role: request.object_role,
            content_type: request.content_type,
            size_bytes: request.body.len() as u64,
            checksum_sha256_hex: request.checksum_sha256_hex,
            etag: None,
            version_id: None,
        })
    }

    async fn head_object(
        &self,
        request: HeadKnowledgeObjectRequest,
    ) -> Result<KnowledgeObjectRef, KnowledgeStorageError> {
        Err(KnowledgeStorageError::NotFound(
            request
                .logical_path
                .unwrap_or_else(|| request.object_key.clone()),
        ))
    }

    async fn get_object_text(
        &self,
        object_ref: &KnowledgeObjectRef,
    ) -> Result<String, KnowledgeStorageError> {
        Err(KnowledgeStorageError::NotFound(
            object_ref.logical_path.clone(),
        ))
    }
}

struct MockRetrievalBackend;

#[async_trait]
impl KnowledgeRetrievalBackend for MockRetrievalBackend {
    async fn search_chunks(
        &self,
        _request: KnowledgeChunkSearchRequest,
    ) -> Result<Vec<KnowledgeChunkSearchHit>, KnowledgeRetrievalBackendError> {
        Ok(Vec::new())
    }
}

struct MockRetrievalTraceStore;

#[async_trait]
impl KnowledgeRetrievalTraceStore for MockRetrievalTraceStore {
    async fn create_trace(
        &self,
        _record: CreateKnowledgeRetrievalTraceRecord,
    ) -> Result<u64, KnowledgeRetrievalTraceStoreError> {
        Ok(1)
    }

    async fn create_hits(
        &self,
        _records: Vec<CreateKnowledgeRetrievalHitRecord>,
    ) -> Result<(), KnowledgeRetrievalTraceStoreError> {
        Ok(())
    }

    async fn retrieve_trace(
        &self,
        _tenant_id: u64,
        _retrieval_trace_id: u64,
    ) -> Result<KnowledgeRetrievalTraceRecord, KnowledgeRetrievalTraceStoreError> {
        Err(KnowledgeRetrievalTraceStoreError::Internal(
            "unsupported in test fake".to_string(),
        ))
    }

    async fn list_trace_hits(
        &self,
        _tenant_id: u64,
        _retrieval_trace_id: u64,
    ) -> Result<Vec<KnowledgeRetrievalTraceHitRecord>, KnowledgeRetrievalTraceStoreError> {
        Ok(Vec::new())
    }
}

struct MockSpaceStore {
    spaces: HashMap<u64, KnowledgeSpace>,
}

#[async_trait]
impl KnowledgeSpaceStore for MockSpaceStore {
    async fn create_space(
        &self,
        _record: CreateKnowledgeSpaceRecord,
    ) -> Result<KnowledgeSpace, KnowledgeSpaceStoreError> {
        Err(KnowledgeSpaceStoreError::Internal(
            "unsupported in test fake".to_string(),
        ))
    }

    async fn get_space(&self, space_id: u64) -> Result<KnowledgeSpace, KnowledgeSpaceStoreError> {
        self.spaces
            .get(&space_id)
            .cloned()
            .ok_or_else(|| KnowledgeSpaceStoreError::Internal("missing space".to_string()))
    }

    async fn mark_drive_space_bound(
        &self,
        _space_id: u64,
        _drive_space_id: String,
    ) -> Result<KnowledgeSpace, KnowledgeSpaceStoreError> {
        Err(KnowledgeSpaceStoreError::Internal(
            "unsupported in test fake".to_string(),
        ))
    }

    async fn mark_okf_bundle_initialized(
        &self,
        _space_id: u64,
    ) -> Result<KnowledgeSpace, KnowledgeSpaceStoreError> {
        Err(KnowledgeSpaceStoreError::Internal(
            "unsupported in test fake".to_string(),
        ))
    }

    async fn mark_space_deleted(&self, _space_id: u64) -> Result<(), KnowledgeSpaceStoreError> {
        Err(KnowledgeSpaceStoreError::Internal(
            "unsupported in test fake".to_string(),
        ))
    }

    async fn update_space(
        &self,
        _space_id: u64,
        _record: sdkwork_intelligence_knowledgebase_service::ports::knowledge_space_store::UpdateKnowledgeSpaceRecord,
    ) -> Result<KnowledgeSpace, KnowledgeSpaceStoreError> {
        Err(KnowledgeSpaceStoreError::Internal(
            "unsupported in test fake".to_string(),
        ))
    }
}

struct MockSourceStore {
    sources: HashMap<u64, Vec<KnowledgeSource>>,
}

#[async_trait]
impl KnowledgeSourceStore for MockSourceStore {
    async fn create_source(
        &self,
        _record: CreateKnowledgeSourceRecord,
    ) -> Result<KnowledgeSource, KnowledgeSourceStoreError> {
        Err(KnowledgeSourceStoreError::Internal(
            "unsupported in test fake".to_string(),
        ))
    }

    async fn list_sources_for_space(
        &self,
        space_id: u64,
    ) -> Result<Vec<KnowledgeSource>, KnowledgeSourceStoreError> {
        Ok(self.sources.get(&space_id).cloned().unwrap_or_default())
    }
}

#[tokio::test]
async fn native_space_mode_is_not_overridden_by_connector_source() {
    let registry = Arc::new(build_default_registry(KnowledgeEngineRuntimeDeps {
        tenant_id: 1,
        okf: OkfNativeKnowledgeEngineDeps::minimal(
            Arc::new(MockOkfConceptStore),
            Arc::new(MockDriveStorage),
        ),
        rag_documents: Arc::new(MockDocumentStore),
        retrieval_backend: Arc::new(MockRetrievalBackend),
        retrieval_traces: Arc::new(MockRetrievalTraceStore),
        rag_index_store: None,
        rag_embedding_store: None,
        rag_embedder: None,
        external_engines: vec![Arc::new(configured_dify_engine())],
    }));

    let resolver = KnowledgeEngineSpaceResolver::new(
        registry,
        Arc::new(MockSpaceStore {
            spaces: HashMap::from([(
                9,
                KnowledgeSpace {
                    id: 9,
                    uuid: "space-9".to_string(),
                    name: "External Space".to_string(),
                    description: None,
                    drive_space_id: Some("drive-9".to_string()),
                    status: sdkwork_knowledgebase_contract::space::KnowledgeSpaceStatus::Active,
                    okf_bundle_initialized: false,
                    knowledge_mode: KnowledgeAgentKnowledgeMode::Rag,
                },
            )]),
        }),
        Arc::new(MockSourceStore {
            sources: HashMap::from([(
                9,
                vec![KnowledgeSource {
                    id: 1,
                    space_id: 9,
                    source_type: KnowledgeSourceType::Connector,
                    provider: Some("dify".to_string()),
                    drive_bucket: None,
                    drive_prefix: None,
                    connector_metadata_json: None,
                }],
            )]),
        }),
    );

    let engine = resolver
        .resolve_for_space(9, None)
        .await
        .expect("resolve native RAG mode");
    assert_eq!(
        engine.descriptor().implementation_id,
        KnowledgeEngineId::RAG_NATIVE
    );
}

#[tokio::test]
async fn explicit_native_override_wins_for_external_space() {
    let registry = Arc::new(build_default_registry(KnowledgeEngineRuntimeDeps {
        tenant_id: 1,
        okf: OkfNativeKnowledgeEngineDeps::minimal(
            Arc::new(MockOkfConceptStore),
            Arc::new(MockDriveStorage),
        ),
        rag_documents: Arc::new(MockDocumentStore),
        retrieval_backend: Arc::new(MockRetrievalBackend),
        retrieval_traces: Arc::new(MockRetrievalTraceStore),
        rag_index_store: None,
        rag_embedding_store: None,
        rag_embedder: None,
        external_engines: vec![Arc::new(configured_dify_engine())],
    }));
    let resolver = KnowledgeEngineSpaceResolver::new(
        registry,
        Arc::new(MockSpaceStore {
            spaces: HashMap::from([(11, external_space(11))]),
        }),
        Arc::new(MockSourceStore {
            sources: HashMap::from([(11, vec![connector_source(1, 11, "dify")])]),
        }),
    );

    let engine = resolver
        .resolve_for_space(11, Some(KnowledgeAgentKnowledgeMode::OkfBundle))
        .await
        .expect("resolve explicit OKF override");
    assert_eq!(
        engine.descriptor().implementation_id,
        KnowledgeEngineId::OKF_NATIVE
    );
}

#[tokio::test]
async fn external_space_rejects_ambiguous_provider_sources() {
    let registry = Arc::new(build_default_registry(KnowledgeEngineRuntimeDeps {
        tenant_id: 1,
        okf: OkfNativeKnowledgeEngineDeps::minimal(
            Arc::new(MockOkfConceptStore),
            Arc::new(MockDriveStorage),
        ),
        rag_documents: Arc::new(MockDocumentStore),
        retrieval_backend: Arc::new(MockRetrievalBackend),
        retrieval_traces: Arc::new(MockRetrievalTraceStore),
        rag_index_store: None,
        rag_embedding_store: None,
        rag_embedder: None,
        external_engines: vec![
            Arc::new(configured_dify_engine()),
            Arc::new(configured_ragflow_engine()),
        ],
    }));
    let resolver = KnowledgeEngineSpaceResolver::new(
        registry,
        Arc::new(MockSpaceStore {
            spaces: HashMap::from([(12, external_space(12))]),
        }),
        Arc::new(MockSourceStore {
            sources: HashMap::from([(
                12,
                vec![
                    connector_source(1, 12, "dify"),
                    connector_source(2, 12, "ragflow"),
                ],
            )]),
        }),
    );

    let error = match resolver.resolve_for_space(12, None).await {
        Ok(_) => panic!("ambiguous external providers must fail"),
        Err(error) => error,
    };
    assert!(matches!(
        error,
        sdkwork_knowledgebase_contract::knowledge_engine::KnowledgeEngineError::Validation(_)
    ));
    assert!(error
        .to_string()
        .contains("explicit provider binding is required"));
}

#[tokio::test]
async fn resolve_for_external_mode_space_requires_connector_provider() {
    let registry = Arc::new(build_default_registry(KnowledgeEngineRuntimeDeps {
        tenant_id: 1,
        okf: OkfNativeKnowledgeEngineDeps::minimal(
            Arc::new(MockOkfConceptStore),
            Arc::new(MockDriveStorage),
        ),
        rag_documents: Arc::new(MockDocumentStore),
        retrieval_backend: Arc::new(MockRetrievalBackend),
        retrieval_traces: Arc::new(MockRetrievalTraceStore),
        rag_index_store: None,
        rag_embedding_store: None,
        rag_embedder: None,
        external_engines: vec![Arc::new(configured_dify_engine())],
    }));

    let external_space = external_space(11);

    let resolver = KnowledgeEngineSpaceResolver::new(
        registry.clone(),
        Arc::new(MockSpaceStore {
            spaces: HashMap::from([(11, external_space.clone())]),
        }),
        Arc::new(MockSourceStore {
            sources: HashMap::from([(
                11,
                vec![KnowledgeSource {
                    id: 2,
                    space_id: 11,
                    source_type: KnowledgeSourceType::Connector,
                    provider: Some("dify".to_string()),
                    drive_bucket: None,
                    drive_prefix: None,
                    connector_metadata_json: None,
                }],
            )]),
        }),
    );

    let engine = resolver
        .resolve_for_space(11, None)
        .await
        .expect("resolve external mode space");
    assert_eq!(
        engine.descriptor().implementation_id,
        KnowledgeEngineId::external("dify").0
    );
    assert_eq!(
        engine.descriptor().agent_provider_id,
        KnowledgeEngineId::external_agent_provider("dify")
    );

    let missing_connector = KnowledgeEngineSpaceResolver::new(
        registry,
        Arc::new(MockSpaceStore {
            spaces: HashMap::from([(11, external_space)]),
        }),
        Arc::new(MockSourceStore {
            sources: HashMap::new(),
        }),
    );
    let result = missing_connector.resolve_for_space(11, None).await;
    assert!(result.is_err());
    assert!(result
        .err()
        .expect("error value")
        .to_string()
        .contains("no external knowledge engine"));
}

fn external_space(space_id: u64) -> KnowledgeSpace {
    KnowledgeSpace {
        id: space_id,
        uuid: format!("space-{space_id}"),
        name: "External Mode Space".to_string(),
        description: None,
        drive_space_id: Some(format!("drive-{space_id}")),
        status: sdkwork_knowledgebase_contract::space::KnowledgeSpaceStatus::Active,
        okf_bundle_initialized: false,
        knowledge_mode: KnowledgeAgentKnowledgeMode::External,
    }
}

fn connector_source(id: u64, space_id: u64, provider: &str) -> KnowledgeSource {
    KnowledgeSource {
        id,
        space_id,
        source_type: KnowledgeSourceType::Connector,
        provider: Some(provider.to_string()),
        drive_bucket: None,
        drive_prefix: None,
        connector_metadata_json: None,
    }
}

fn configured_dify_engine() -> DifyKnowledgeEngine {
    DifyKnowledgeEngine::with_config(
        DifyConnectorConfig {
            base_url: "http://127.0.0.1:1/v1".to_string(),
            api_key: "test-only".to_string(),
            default_dataset_id: None,
        },
        None,
    )
}

fn configured_ragflow_engine() -> RagflowKnowledgeEngine {
    RagflowKnowledgeEngine::with_config(
        RagflowConnectorConfig {
            base_url: "http://127.0.0.1:1/api/v1".to_string(),
            api_key: "test-only".to_string(),
            default_dataset_id: None,
        },
        None,
    )
}
