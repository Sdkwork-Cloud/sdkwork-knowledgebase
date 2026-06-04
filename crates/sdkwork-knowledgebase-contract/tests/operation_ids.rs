use sdkwork_knowledgebase_contract::operations::{
    ALL_OPERATION_IDS, DOCUMENTS_CREATE, DOCUMENTS_LIST, DOCUMENTS_RETRIEVE,
    DOCUMENTS_VERSIONS_CREATE, DRIVE_IMPORTS_CREATE, INGESTS_CREATE, INGESTS_RETRIEVE,
    SOURCES_CREATE, SOURCES_LIST, SPACES_BROWSER_LIST, WIKI_INDEX_REBUILD, WIKI_INDEX_RETRIEVE,
    WIKI_LOG_ENTRIES_CREATE, WIKI_SCHEMA_PROFILES_CREATE,
};

#[test]
fn wiki_operation_ids_are_nested_under_wiki_resource() {
    assert_eq!(WIKI_INDEX_RETRIEVE, "wiki.index.retrieve");
    assert_eq!(WIKI_INDEX_REBUILD, "wiki.index.rebuild");
    assert_eq!(WIKI_LOG_ENTRIES_CREATE, "wiki.log.entries.create");
    assert_eq!(WIKI_SCHEMA_PROFILES_CREATE, "wiki.schema.profiles.create");
}

#[test]
fn operation_ids_follow_sdkwork_dotted_style() {
    assert!(!ALL_OPERATION_IDS.iter().any(|id| id.contains('_')));
    assert!(!ALL_OPERATION_IDS
        .iter()
        .any(|id| id.starts_with("wikiIndex")));
    assert!(!ALL_OPERATION_IDS
        .iter()
        .any(|id| id.starts_with("wikiPages")));
    assert!(
        ALL_OPERATION_IDS
            .iter()
            .filter(|id| id.starts_with("wiki."))
            .count()
            >= 6
    );
}

#[test]
fn source_document_ingest_operation_ids_follow_sdkwork_resource_tree() {
    assert_eq!(SOURCES_LIST, "sources.list");
    assert_eq!(SOURCES_CREATE, "sources.create");
    assert_eq!(DOCUMENTS_LIST, "documents.list");
    assert_eq!(DOCUMENTS_CREATE, "documents.create");
    assert_eq!(DOCUMENTS_RETRIEVE, "documents.retrieve");
    assert_eq!(DOCUMENTS_VERSIONS_CREATE, "documents.versions.create");
    assert_eq!(DRIVE_IMPORTS_CREATE, "driveImports.create");
    assert_eq!(INGESTS_CREATE, "ingests.create");
    assert_eq!(INGESTS_RETRIEVE, "ingests.retrieve");
    assert_eq!(SPACES_BROWSER_LIST, "spaces.browser.list");
}
