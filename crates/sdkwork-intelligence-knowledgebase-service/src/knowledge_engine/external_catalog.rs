//! External knowledge engines registered from `external/knowledge-engines/catalog.manifest.json`.
//!
//! Catalog and stub tier vendors register metadata-driven stub engines until an approved
//! adapter crate (`crates/sdkwork-knowledgebase-engine-{vendorId}`) ships.
//! Adapter-tier vendors are excluded here; runtime wires their adapter crates instead.

use async_trait::async_trait;
use sdkwork_knowledgebase_contract::knowledge_engine::{
    descriptor_for_external, KnowledgeEngineDescriptor, KnowledgeEngineDocument,
    KnowledgeEngineDocumentList, KnowledgeEngineError, KnowledgeEngineHealth,
    KnowledgeEngineHealthStatus, KnowledgeEngineListRequest, KnowledgeEngineReadRequest,
    KnowledgeEngineSearchRequest, KnowledgeEngineSearchResult,
};
use serde::Deserialize;
use std::collections::HashSet;
use std::sync::Arc;

use super::KnowledgeEngine;
use crate::ports::knowledge_engine::ExternalKnowledgeEngine;

const CATALOG_MANIFEST: &str =
    include_str!("../../../../external/knowledge-engines/catalog.manifest.json");

#[derive(Debug, Deserialize)]
struct CatalogManifest {
    vendors: Vec<CatalogVendorEntry>,
}

#[derive(Debug, Deserialize)]
struct CatalogVendorEntry {
    #[serde(rename = "vendorId")]
    vendor_id: String,
    #[serde(rename = "manifestPath")]
    _manifest_path: String,
    #[serde(rename = "implementationId")]
    implementation_id: String,
    #[serde(rename = "integrationTier")]
    _integration_tier: String,
}

#[derive(Debug, Deserialize)]
struct VendorManifest {
    #[serde(rename = "vendorId")]
    vendor_id: String,
    #[serde(rename = "displayName")]
    display_name: String,
    #[serde(rename = "implementationId")]
    implementation_id: String,
    #[serde(rename = "agentProviderId")]
    _agent_provider_id: String,
    #[serde(rename = "integrationTier")]
    integration_tier: String,
}

pub struct CatalogExternalKnowledgeEngine {
    descriptor: KnowledgeEngineDescriptor,
    stub_message: String,
}

impl CatalogExternalKnowledgeEngine {
    fn from_vendor_manifest(manifest: &VendorManifest) -> Self {
        let stub_message = catalog_stub_message(&manifest.integration_tier, &manifest.display_name);
        Self {
            descriptor: descriptor_for_external(&manifest.vendor_id, &manifest.display_name),
            stub_message,
        }
    }
}

pub fn load_external_engines_from_catalog() -> Vec<Arc<dyn KnowledgeEngine>> {
    let catalog: CatalogManifest =
        serde_json::from_str(CATALOG_MANIFEST).expect("catalog.manifest.json must parse");

    let mut engines = Vec::new();
    let mut seen = HashSet::new();

    for entry in catalog.vendors {
        if !seen.insert(entry.implementation_id.clone()) {
            continue;
        }

        let Some(manifest_json) = vendor_manifest_json(&entry.vendor_id) else {
            continue;
        };
        let manifest: VendorManifest =
            serde_json::from_str(manifest_json).unwrap_or_else(|error| {
                panic!(
                    "vendor manifest for {} must parse: {error}",
                    entry.vendor_id
                )
            });
        assert_eq!(
            manifest.implementation_id, entry.implementation_id,
            "catalog entry implementationId must match vendor manifest for {}",
            entry.vendor_id
        );
        if manifest.integration_tier == "adapter" {
            continue;
        }
        engines.push(
            Arc::new(CatalogExternalKnowledgeEngine::from_vendor_manifest(
                &manifest,
            )) as Arc<dyn KnowledgeEngine>,
        );
    }

    engines
}

fn catalog_stub_message(integration_tier: &str, display_name: &str) -> String {
    match integration_tier {
        "stub" => format!(
            "{display_name} adapter is registered at stub tier; configure kb_source connector before use"
        ),
        "adapter" => format!(
            "{display_name} adapter is registered at adapter tier; configure connector env or kb_source metadata before use"
        ),
        "catalog" => format!(
            "{display_name} is catalog-registered; approved adapter crate wiring is required before use"
        ),
        other => format!(
            "{display_name} external engine is registered at integration tier {other}; connector wiring is required"
        ),
    }
}

fn vendor_manifest_json(vendor_id: &str) -> Option<&'static str> {
    match vendor_id {
        "dify" => Some(include_str!(
            "../../../../external/knowledge-engines/vendors/dify/engine.manifest.json"
        )),
        "ragflow" => Some(include_str!(
            "../../../../external/knowledge-engines/vendors/ragflow/engine.manifest.json"
        )),
        "onyx" => Some(include_str!(
            "../../../../external/knowledge-engines/vendors/onyx/engine.manifest.json"
        )),
        "anythingllm" => Some(include_str!(
            "../../../../external/knowledge-engines/vendors/anythingllm/engine.manifest.json"
        )),
        "open-webui" => Some(include_str!(
            "../../../../external/knowledge-engines/vendors/open-webui/engine.manifest.json"
        )),
        "flowise" => Some(include_str!(
            "../../../../external/knowledge-engines/vendors/flowise/engine.manifest.json"
        )),
        "langchain" => Some(include_str!(
            "../../../../external/knowledge-engines/vendors/langchain/engine.manifest.json"
        )),
        "llamaindex" => Some(include_str!(
            "../../../../external/knowledge-engines/vendors/llamaindex/engine.manifest.json"
        )),
        "haystack" => Some(include_str!(
            "../../../../external/knowledge-engines/vendors/haystack/engine.manifest.json"
        )),
        "qdrant" => Some(include_str!(
            "../../../../external/knowledge-engines/vendors/qdrant/engine.manifest.json"
        )),
        "weaviate" => Some(include_str!(
            "../../../../external/knowledge-engines/vendors/weaviate/engine.manifest.json"
        )),
        "chroma" => Some(include_str!(
            "../../../../external/knowledge-engines/vendors/chroma/engine.manifest.json"
        )),
        _ => None,
    }
}

#[async_trait]
impl KnowledgeEngine for CatalogExternalKnowledgeEngine {
    fn descriptor(&self) -> KnowledgeEngineDescriptor {
        self.descriptor.clone()
    }

    async fn health(&self) -> Result<KnowledgeEngineHealth, KnowledgeEngineError> {
        Ok(KnowledgeEngineHealth {
            implementation_id: self.descriptor.implementation_id.clone(),
            status: KnowledgeEngineHealthStatus::Degraded,
            detail: Some(self.stub_message.clone()),
        })
    }

    async fn search(
        &self,
        _request: KnowledgeEngineSearchRequest,
    ) -> Result<KnowledgeEngineSearchResult, KnowledgeEngineError> {
        Err(KnowledgeEngineError::Unsupported(self.stub_message.clone()))
    }

    async fn read_document(
        &self,
        _request: KnowledgeEngineReadRequest,
    ) -> Result<KnowledgeEngineDocument, KnowledgeEngineError> {
        Err(KnowledgeEngineError::Unsupported(self.stub_message.clone()))
    }

    async fn list_documents(
        &self,
        _request: KnowledgeEngineListRequest,
    ) -> Result<KnowledgeEngineDocumentList, KnowledgeEngineError> {
        Ok(KnowledgeEngineDocumentList { items: Vec::new() })
    }
}

#[async_trait]
impl ExternalKnowledgeEngine for CatalogExternalKnowledgeEngine {
    async fn connector_health(&self) -> Result<KnowledgeEngineHealth, KnowledgeEngineError> {
        self.health().await
    }

    async fn sync_sources(&self, _space_id: u64) -> Result<u32, KnowledgeEngineError> {
        Err(KnowledgeEngineError::Unsupported(self.stub_message.clone()))
    }
}
