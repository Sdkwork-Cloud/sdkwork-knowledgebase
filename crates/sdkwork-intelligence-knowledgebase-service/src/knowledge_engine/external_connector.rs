//! Shared external connector resolution for adapter-tier knowledge engines.

use sdkwork_knowledgebase_contract::knowledge_engine::{
    implementation_id_from_provider, KnowledgeEngineError,
};
use sdkwork_knowledgebase_contract::source::{
    dataset_id_from_connector_metadata_json, KnowledgeSourceType,
};

use crate::ports::knowledge_source_store::KnowledgeSourceStore;

/// Resolves the upstream dataset/knowledge-base id for a space from connector sources or env default.
pub async fn resolve_connector_dataset_id_for_space(
    source_store: &dyn KnowledgeSourceStore,
    space_id: u64,
    expected_implementation_id: &str,
    default_dataset_id: Option<String>,
    config_hint: &str,
) -> Result<String, KnowledgeEngineError> {
    let sources = source_store
        .list_sources_for_space(space_id)
        .await
        .map_err(|error| KnowledgeEngineError::Internal(error.to_string()))?;

    for source in sources {
        if source.source_type != KnowledgeSourceType::Connector {
            continue;
        }
        let Some(provider) = source.provider.as_deref() else {
            continue;
        };
        let Some(implementation_id) = implementation_id_from_provider(provider) else {
            continue;
        };
        if implementation_id != expected_implementation_id {
            continue;
        }
        if let Some(dataset_id) =
            dataset_id_from_connector_metadata_json(source.connector_metadata_json.as_deref())
        {
            return Ok(dataset_id);
        }
    }

    default_dataset_id.ok_or_else(|| {
        KnowledgeEngineError::Validation(format!(
            "external search requires kb_source connector metadata datasetId or {config_hint} for space_id={space_id}"
        ))
    })
}
