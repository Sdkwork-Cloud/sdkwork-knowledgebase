use crate::ports::knowledge_okf_concept_store::{
    AppendKnowledgeOkfLogEntryRecord, KnowledgeOkfConceptStore, KnowledgeOkfConceptStoreError,
};
use time::{format_description::well_known::Rfc3339, OffsetDateTime};

pub(crate) async fn append_okf_bundle_log_entry(
    concept_store: &dyn KnowledgeOkfConceptStore,
    space_id: u64,
    event_type: impl Into<String>,
    title: impl Into<String>,
    actor: &str,
    affected_concepts: Vec<String>,
    warnings: Vec<String>,
) -> Result<(), KnowledgeOkfConceptStoreError> {
    concept_store
        .append_log_entry(AppendKnowledgeOkfLogEntryRecord {
            space_id,
            event_type: event_type.into(),
            event_time: now_rfc3339()?,
            title: title.into(),
            actor: actor.to_string(),
            affected_concepts,
            audit_event_id: None,
            warnings,
            privacy_level: "internal".to_string(),
        })
        .await?;
    Ok(())
}

fn now_rfc3339() -> Result<String, KnowledgeOkfConceptStoreError> {
    OffsetDateTime::now_utc()
        .format(&Rfc3339)
        .map_err(|error| KnowledgeOkfConceptStoreError::Internal(error.to_string()))
}
