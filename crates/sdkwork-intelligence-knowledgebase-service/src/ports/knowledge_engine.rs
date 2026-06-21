//! Knowledge Engine SPI — switchable product backends (OKF native, RAG native, external).
//!
//! Contract types: `sdkwork_knowledgebase_contract::knowledge_engine`.
//! Machine-readable spec: `specs/knowledge-engine-spi.spec.json`.

use async_trait::async_trait;
use sdkwork_knowledgebase_contract::knowledge_engine::{
    descriptor_for_mode, KnowledgeEngineDescriptor, KnowledgeEngineDocument,
    KnowledgeEngineDocumentList, KnowledgeEngineError, KnowledgeEngineHealth, KnowledgeEngineId,
    KnowledgeEngineListRequest, KnowledgeEngineReadRequest, KnowledgeEngineSearchRequest,
    KnowledgeEngineSearchResult,
};
use sdkwork_knowledgebase_contract::okf::{OkfBundleLintResult, OkfConceptSummary};
use sdkwork_knowledgebase_contract::rag::{
    KnowledgeAgentKnowledgeMode, KnowledgeContextPack, KnowledgeContextPackRequest,
    KnowledgeRetrievalRequest, KnowledgeRetrievalResult,
};
use std::collections::HashMap;
use std::sync::Arc;

/// Core SPI implemented by every knowledge engine (native or third-party).
#[async_trait]
pub trait KnowledgeEngine: Send + Sync {
    fn descriptor(&self) -> KnowledgeEngineDescriptor;

    async fn health(&self) -> Result<KnowledgeEngineHealth, KnowledgeEngineError>;

    async fn search(
        &self,
        request: KnowledgeEngineSearchRequest,
    ) -> Result<KnowledgeEngineSearchResult, KnowledgeEngineError>;

    async fn read_document(
        &self,
        request: KnowledgeEngineReadRequest,
    ) -> Result<KnowledgeEngineDocument, KnowledgeEngineError>;

    async fn list_documents(
        &self,
        request: KnowledgeEngineListRequest,
    ) -> Result<KnowledgeEngineDocumentList, KnowledgeEngineError>;
}

/// Spec-aligned registration surface (`registry.register` in `knowledge-engine-spi.spec.json`).
pub trait KnowledgeEngineRegistrar: Send + Sync {
    fn register(&mut self, engine: Arc<dyn KnowledgeEngine>);
}

/// Resolves and registers knowledge engines for a tenant/runtime.
pub trait KnowledgeEngineRegistry: Send + Sync {
    fn resolve_for_mode(
        &self,
        mode: KnowledgeAgentKnowledgeMode,
    ) -> Result<Arc<dyn KnowledgeEngine>, KnowledgeEngineError>;

    fn resolve_by_id(
        &self,
        implementation_id: &str,
    ) -> Result<Arc<dyn KnowledgeEngine>, KnowledgeEngineError>;

    fn list_registered(&self) -> Vec<KnowledgeEngineDescriptor>;
}

/// Async per-space resolution surface aligned with `knowledge-engine-spi.spec.json` registry.resolve_for_space.
#[async_trait]
pub trait KnowledgeEngineSpaceRegistry: Send + Sync {
    async fn resolve_for_space(
        &self,
        space_id: u64,
        mode_override: Option<KnowledgeAgentKnowledgeMode>,
    ) -> Result<Arc<dyn KnowledgeEngine>, KnowledgeEngineError>;
}

#[derive(Default)]
pub struct InMemoryKnowledgeEngineRegistry {
    engines: HashMap<String, Arc<dyn KnowledgeEngine>>,
}

impl InMemoryKnowledgeEngineRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(&mut self, engine: Arc<dyn KnowledgeEngine>) {
        KnowledgeEngineRegistrar::register(self, engine);
    }
}

impl KnowledgeEngineRegistrar for InMemoryKnowledgeEngineRegistry {
    fn register(&mut self, engine: Arc<dyn KnowledgeEngine>) {
        let id = engine.descriptor().implementation_id.clone();
        self.engines.insert(id, engine);
    }
}

impl KnowledgeEngineRegistry for InMemoryKnowledgeEngineRegistry {
    fn resolve_for_mode(
        &self,
        mode: KnowledgeAgentKnowledgeMode,
    ) -> Result<Arc<dyn KnowledgeEngine>, KnowledgeEngineError> {
        if mode == KnowledgeAgentKnowledgeMode::External {
            return Err(KnowledgeEngineError::Unsupported(
                "external knowledge mode must be resolved per space via connector provider"
                    .to_string(),
            ));
        }
        let descriptor = descriptor_for_mode(mode);
        self.resolve_by_id(&descriptor.implementation_id)
    }

    fn resolve_by_id(
        &self,
        implementation_id: &str,
    ) -> Result<Arc<dyn KnowledgeEngine>, KnowledgeEngineError> {
        self.engines.get(implementation_id).cloned().ok_or_else(|| {
            KnowledgeEngineError::NotFound(format!(
                "no knowledge engine registered for implementation_id={implementation_id}"
            ))
        })
    }

    fn list_registered(&self) -> Vec<KnowledgeEngineDescriptor> {
        self.engines
            .values()
            .map(|engine| engine.descriptor())
            .collect()
    }
}

pub fn default_implementation_id_for_mode(mode: KnowledgeAgentKnowledgeMode) -> &'static str {
    match mode {
        KnowledgeAgentKnowledgeMode::OkfBundle => KnowledgeEngineId::OKF_NATIVE,
        KnowledgeAgentKnowledgeMode::Rag => KnowledgeEngineId::RAG_NATIVE,
        KnowledgeAgentKnowledgeMode::External => "engine.knowledge.external.unresolved",
    }
}

/// Extension SPI for native OKF bundle operations.
#[async_trait]
pub trait OkfBundleEngine: KnowledgeEngine {
    async fn list_concepts(
        &self,
        space_id: u64,
    ) -> Result<Vec<OkfConceptSummary>, KnowledgeEngineError>;

    async fn upsert_concept(
        &self,
        _request: sdkwork_knowledgebase_contract::okf::OkfConceptUpsertRequest,
    ) -> Result<sdkwork_knowledgebase_contract::okf::KnowledgeOkfConcept, KnowledgeEngineError>
    {
        Err(KnowledgeEngineError::Unsupported(
            "okf upsert_concept requires hosted OkfConceptService wiring".to_string(),
        ))
    }

    async fn delete_concept(
        &self,
        _space_id: u64,
        _concept_row_id: u64,
        _actor: &str,
    ) -> Result<(), KnowledgeEngineError> {
        Err(KnowledgeEngineError::Unsupported(
            "okf delete_concept requires hosted OkfConceptService wiring".to_string(),
        ))
    }

    async fn publish_concept(
        &self,
        _request: sdkwork_knowledgebase_contract::okf::PublishKnowledgeOkfConceptRequest,
    ) -> Result<
        sdkwork_knowledgebase_contract::okf::KnowledgeOkfConceptPublication,
        KnowledgeEngineError,
    > {
        Err(KnowledgeEngineError::Unsupported(
            "okf publish_concept requires hosted OkfConceptService wiring".to_string(),
        ))
    }

    async fn lint_bundle(&self, space_id: u64) -> Result<(), KnowledgeEngineError> {
        let report = self.lint_bundle_report(space_id).await?;
        if report.conformance != "pass" {
            return Err(KnowledgeEngineError::Validation(format!(
                "okf bundle lint failed with {} issue(s)",
                report.issues.len()
            )));
        }
        Ok(())
    }

    async fn lint_bundle_report(
        &self,
        _space_id: u64,
    ) -> Result<OkfBundleLintResult, KnowledgeEngineError> {
        Err(KnowledgeEngineError::Unsupported(
            "okf lint_bundle_report requires hosted OkfBundleLinterService wiring".to_string(),
        ))
    }

    async fn import_bundle(
        &self,
        _request: sdkwork_knowledgebase_contract::okf::OkfBundleImportRequest,
    ) -> Result<sdkwork_knowledgebase_contract::okf::OkfBundleImportResult, KnowledgeEngineError>
    {
        Err(KnowledgeEngineError::Unsupported(
            "okf import_bundle requires hosted OkfBundleImporterService wiring".to_string(),
        ))
    }

    async fn export_bundle(
        &self,
        _request: sdkwork_knowledgebase_contract::okf::OkfBundleExportRequest,
    ) -> Result<
        sdkwork_knowledgebase_contract::okf_bundle_file::KnowledgeOkfBundleFile,
        KnowledgeEngineError,
    > {
        Err(KnowledgeEngineError::Unsupported(
            "okf export_bundle requires hosted OkfBundleExporterService wiring".to_string(),
        ))
    }

    async fn rebuild_index(&self, _space_id: u64) -> Result<(), KnowledgeEngineError> {
        Err(KnowledgeEngineError::Unsupported(
            "okf rebuild_index requires hosted index rebuild wiring".to_string(),
        ))
    }
}

/// Extension SPI for native RAG retrieval operations.
#[async_trait]
pub trait RagKnowledgeEngine: KnowledgeEngine {
    async fn retrieve(
        &self,
        request: KnowledgeRetrievalRequest,
    ) -> Result<KnowledgeRetrievalResult, KnowledgeEngineError>;

    async fn create_context_pack(
        &self,
        request: KnowledgeContextPackRequest,
    ) -> Result<KnowledgeContextPack, KnowledgeEngineError>;

    async fn rebuild_index(&self, _space_id: u64) -> Result<(), KnowledgeEngineError> {
        Err(KnowledgeEngineError::Unsupported(
            "rag rebuild_index requires hosted embedding index wiring".to_string(),
        ))
    }
}

/// Extension SPI for third-party knowledge backends (connector health, source sync).
#[async_trait]
pub trait ExternalKnowledgeEngine: KnowledgeEngine {
    async fn connector_health(&self) -> Result<KnowledgeEngineHealth, KnowledgeEngineError>;

    async fn sync_sources(&self, space_id: u64) -> Result<u32, KnowledgeEngineError>;
}
