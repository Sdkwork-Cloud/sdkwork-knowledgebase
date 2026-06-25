use sdkwork_knowledgebase_contract::knowledge_engine::KnowledgeEngineDocumentRef;

pub fn parse_namespace_space_id(namespace: Option<&str>) -> Result<u64, String> {
    let Some(value) = namespace else {
        return Err(
            "knowledge namespace is required and must contain a numeric space id".to_string(),
        );
    };

    let value = value.strip_prefix("space:").unwrap_or(value);
    value
        .parse::<u64>()
        .map_err(|_| "knowledge namespace must be a numeric space id".to_string())
}

pub fn format_scoped_document_id(space_id: u64, document_id: &str) -> String {
    format!("{space_id}/{document_id}")
}

pub fn parse_scoped_document_id(document_id: &str) -> Option<(u64, String)> {
    let (space_id, scoped_id) = document_id.split_once('/')?;
    Some((space_id.parse().ok()?, scoped_id.to_string()))
}

pub fn scoped_document_refs(
    space_id: u64,
    items: Vec<KnowledgeEngineDocumentRef>,
) -> Vec<(String, KnowledgeEngineDocumentRef)> {
    items
        .into_iter()
        .map(|item| (format_scoped_document_id(space_id, &item.document_id), item))
        .collect()
}
