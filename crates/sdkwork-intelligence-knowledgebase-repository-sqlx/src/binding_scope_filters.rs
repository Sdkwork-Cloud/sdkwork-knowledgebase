//! Applies `KnowledgeRetrievalBinding` source/document filters to chunk search SQL.

use sdkwork_intelligence_knowledgebase_service::ports::knowledge_retrieval_backend::KnowledgeRetrievalBackendError;
use sdkwork_knowledgebase_contract::rag::{KnowledgeFilter, KnowledgeRetrievalBinding};
use sqlx::{Any, Postgres, QueryBuilder};

const ACTIVE_STATUS: i64 = 1;

pub fn push_binding_scope_filters(
    query: &mut QueryBuilder<'_, Any>,
    tenant_id: i64,
    space_id: i64,
    binding: &KnowledgeRetrievalBinding,
) -> Result<(), KnowledgeRetrievalBackendError> {
    if let Some(filters) = binding.source_filter.as_ref() {
        for filter in filters {
            push_source_scope_filter(query, tenant_id, space_id, filter)?;
        }
    }
    if let Some(filters) = binding.document_filter.as_ref() {
        for filter in filters {
            push_document_scope_filter(query, filter)?;
        }
    }
    Ok(())
}

pub fn push_binding_scope_filters_postgres(
    query: &mut QueryBuilder<'_, Postgres>,
    tenant_id: i64,
    space_id: i64,
    binding: &KnowledgeRetrievalBinding,
) -> Result<(), KnowledgeRetrievalBackendError> {
    if let Some(filters) = binding.source_filter.as_ref() {
        for filter in filters {
            push_source_scope_filter_postgres(query, tenant_id, space_id, filter)?;
        }
    }
    if let Some(filters) = binding.document_filter.as_ref() {
        for filter in filters {
            push_document_scope_filter_postgres(query, filter)?;
        }
    }
    Ok(())
}

fn push_source_scope_filter(
    query: &mut QueryBuilder<'_, Any>,
    tenant_id: i64,
    space_id: i64,
    filter: &KnowledgeFilter,
) -> Result<(), KnowledgeRetrievalBackendError> {
    match classify_source_filter_key(&filter.key) {
        SourceFilterKey::SourceType => {
            query.push(" AND d.source_id IN (SELECT id FROM kb_source WHERE tenant_id = ");
            query.push_bind(tenant_id);
            query.push(" AND space_id = ");
            query.push_bind(space_id);
            query.push(" AND source_type = ");
            query.push_bind(filter.value.clone());
            query.push(" AND status = ");
            query.push_bind(ACTIVE_STATUS);
            query.push(")");
        }
        SourceFilterKey::Provider => {
            query.push(" AND d.source_id IN (SELECT id FROM kb_source WHERE tenant_id = ");
            query.push_bind(tenant_id);
            query.push(" AND space_id = ");
            query.push_bind(space_id);
            query.push(" AND provider = ");
            query.push_bind(filter.value.clone());
            query.push(" AND status = ");
            query.push_bind(ACTIVE_STATUS);
            query.push(")");
        }
        SourceFilterKey::SourceId => {
            let source_id = parse_filter_u64("sourceId", &filter.value)?;
            query.push(" AND d.source_id = ");
            query.push_bind(source_id);
        }
        SourceFilterKey::Unsupported(key) => {
            return Err(unsupported_binding_filter_key(key));
        }
    }
    Ok(())
}

fn push_source_scope_filter_postgres(
    query: &mut QueryBuilder<'_, Postgres>,
    tenant_id: i64,
    space_id: i64,
    filter: &KnowledgeFilter,
) -> Result<(), KnowledgeRetrievalBackendError> {
    match classify_source_filter_key(&filter.key) {
        SourceFilterKey::SourceType => {
            query.push(" AND d.source_id IN (SELECT id FROM kb_source WHERE tenant_id = ");
            query.push_bind(tenant_id);
            query.push(" AND space_id = ");
            query.push_bind(space_id);
            query.push(" AND source_type = ");
            query.push_bind(filter.value.clone());
            query.push(" AND status = ");
            query.push_bind(ACTIVE_STATUS);
            query.push(")");
        }
        SourceFilterKey::Provider => {
            query.push(" AND d.source_id IN (SELECT id FROM kb_source WHERE tenant_id = ");
            query.push_bind(tenant_id);
            query.push(" AND space_id = ");
            query.push_bind(space_id);
            query.push(" AND provider = ");
            query.push_bind(filter.value.clone());
            query.push(" AND status = ");
            query.push_bind(ACTIVE_STATUS);
            query.push(")");
        }
        SourceFilterKey::SourceId => {
            let source_id = parse_filter_u64("sourceId", &filter.value)?;
            query.push(" AND d.source_id = ");
            query.push_bind(source_id);
        }
        SourceFilterKey::Unsupported(key) => {
            return Err(unsupported_binding_filter_key(key));
        }
    }
    Ok(())
}

fn push_document_scope_filter(
    query: &mut QueryBuilder<'_, Any>,
    filter: &KnowledgeFilter,
) -> Result<(), KnowledgeRetrievalBackendError> {
    match classify_document_filter_key(&filter.key) {
        DocumentFilterKey::DocumentId => {
            let document_id = parse_filter_u64("documentId", &filter.value)?;
            query.push(" AND d.id = ");
            query.push_bind(document_id);
        }
        DocumentFilterKey::Language => {
            query.push(" AND d.language = ");
            query.push_bind(filter.value.clone());
        }
        DocumentFilterKey::MimeType => {
            query.push(" AND d.mime_type = ");
            query.push_bind(filter.value.clone());
        }
        DocumentFilterKey::Visibility => {
            let visibility = parse_filter_u64("visibility", &filter.value)?;
            query.push(" AND d.visibility = ");
            query.push_bind(visibility);
        }
        DocumentFilterKey::Metadata(path) => {
            query.push(" AND json_extract(d.metadata, ");
            query.push_bind(format!("$.{path}"));
            query.push(") = ");
            query.push_bind(filter.value.clone());
        }
        DocumentFilterKey::Unsupported(key) => {
            return Err(unsupported_binding_filter_key(key));
        }
    }
    Ok(())
}

fn push_document_scope_filter_postgres(
    query: &mut QueryBuilder<'_, Postgres>,
    filter: &KnowledgeFilter,
) -> Result<(), KnowledgeRetrievalBackendError> {
    match classify_document_filter_key(&filter.key) {
        DocumentFilterKey::DocumentId => {
            let document_id = parse_filter_u64("documentId", &filter.value)?;
            query.push(" AND d.id = ");
            query.push_bind(document_id);
        }
        DocumentFilterKey::Language => {
            query.push(" AND d.language = ");
            query.push_bind(filter.value.clone());
        }
        DocumentFilterKey::MimeType => {
            query.push(" AND d.mime_type = ");
            query.push_bind(filter.value.clone());
        }
        DocumentFilterKey::Visibility => {
            let visibility = parse_filter_u64("visibility", &filter.value)?;
            query.push(" AND d.visibility = ");
            query.push_bind(visibility);
        }
        DocumentFilterKey::Metadata(path) => {
            query.push(" AND d.metadata::jsonb ->> ");
            query.push_bind(path);
            query.push(" = ");
            query.push_bind(filter.value.clone());
        }
        DocumentFilterKey::Unsupported(key) => {
            return Err(unsupported_binding_filter_key(key));
        }
    }
    Ok(())
}

enum SourceFilterKey {
    SourceType,
    Provider,
    SourceId,
    Unsupported(String),
}

enum DocumentFilterKey {
    DocumentId,
    Language,
    MimeType,
    Visibility,
    Metadata(String),
    Unsupported(String),
}

fn classify_source_filter_key(key: &str) -> SourceFilterKey {
    match normalize_filter_key(key).as_str() {
        "source_type" | "sourcetype" => SourceFilterKey::SourceType,
        "provider" => SourceFilterKey::Provider,
        "source_id" | "sourceid" => SourceFilterKey::SourceId,
        _ => SourceFilterKey::Unsupported(key.to_string()),
    }
}

fn classify_document_filter_key(key: &str) -> DocumentFilterKey {
    if let Some(path) = key.strip_prefix("metadata.") {
        if path.is_empty() {
            return DocumentFilterKey::Unsupported(key.to_string());
        }
        return DocumentFilterKey::Metadata(path.to_string());
    }

    match normalize_filter_key(key).as_str() {
        "document_id" | "documentid" => DocumentFilterKey::DocumentId,
        "language" => DocumentFilterKey::Language,
        "mime_type" | "mimetype" => DocumentFilterKey::MimeType,
        "visibility" => DocumentFilterKey::Visibility,
        _ => DocumentFilterKey::Unsupported(key.to_string()),
    }
}

fn normalize_filter_key(key: &str) -> String {
    key.trim().to_ascii_lowercase().replace('-', "_")
}

fn parse_filter_u64(field: &str, value: &str) -> Result<i64, KnowledgeRetrievalBackendError> {
    let parsed = value.trim().parse::<u64>().map_err(|_| {
        KnowledgeRetrievalBackendError::Internal(format!(
            "binding filter {field} expects a numeric value"
        ))
    })?;
    i64::try_from(parsed).map_err(|_| {
        KnowledgeRetrievalBackendError::Internal(format!(
            "binding filter {field} exceeds i64 range"
        ))
    })
}

fn unsupported_binding_filter_key(key: String) -> KnowledgeRetrievalBackendError {
    KnowledgeRetrievalBackendError::Internal(format!(
        "unsupported knowledge retrieval binding filter key: {key}"
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classifies_contract_source_and_document_filter_keys() {
        assert!(matches!(
            classify_source_filter_key("sourceType"),
            SourceFilterKey::SourceType
        ));
        assert!(matches!(
            classify_document_filter_key("language"),
            DocumentFilterKey::Language
        ));
        assert!(matches!(
            classify_document_filter_key("metadata.locale"),
            DocumentFilterKey::Metadata(path) if path == "locale"
        ));
    }
}
