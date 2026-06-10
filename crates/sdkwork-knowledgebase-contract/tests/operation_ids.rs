use sdkwork_knowledgebase_contract::operations::{
    AGENT_PROFILES_BINDINGS_CREATE, AGENT_PROFILES_BINDINGS_DELETE, AGENT_PROFILES_BINDINGS_LIST,
    AGENT_PROFILES_BINDINGS_UPDATE, AGENT_PROFILES_CREATE, AGENT_PROFILES_DELETE,
    AGENT_PROFILES_RETRIEVAL_PREVIEW_CREATE, AGENT_PROFILES_RETRIEVE, AGENT_PROFILES_UPDATE,
    ALL_OPERATION_IDS, CONTEXT_PACKS_CREATE, DOCUMENTS_CREATE, DOCUMENTS_LIST, DOCUMENTS_RETRIEVE,
    DOCUMENTS_VERSIONS_CREATE, DRIVE_IMPORTS_CREATE, INGESTS_CREATE, INGESTS_RETRIEVE,
    RETRIEVALS_CREATE, RETRIEVALS_RETRIEVE, SOURCES_CREATE, SOURCES_LIST, SPACES_BROWSER_LIST,
    WIKI_INDEX_REBUILD, WIKI_INDEX_RETRIEVE, WIKI_LOG_ENTRIES_CREATE, WIKI_SCHEMA_PROFILES_CREATE,
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

#[test]
fn rag_and_knowledge_agent_operation_ids_follow_sdkwork_resource_tree() {
    assert_eq!(RETRIEVALS_CREATE, "retrievals.create");
    assert_eq!(RETRIEVALS_RETRIEVE, "retrievals.retrieve");
    assert_eq!(CONTEXT_PACKS_CREATE, "contextPacks.create");
    assert_eq!(AGENT_PROFILES_CREATE, "agentProfiles.create");
    assert_eq!(AGENT_PROFILES_RETRIEVE, "agentProfiles.retrieve");
    assert_eq!(AGENT_PROFILES_UPDATE, "agentProfiles.update");
    assert_eq!(AGENT_PROFILES_DELETE, "agentProfiles.delete");
    assert_eq!(AGENT_PROFILES_BINDINGS_LIST, "agentProfiles.bindings.list");
    assert_eq!(
        AGENT_PROFILES_BINDINGS_CREATE,
        "agentProfiles.bindings.create"
    );
    assert_eq!(
        AGENT_PROFILES_BINDINGS_UPDATE,
        "agentProfiles.bindings.update"
    );
    assert_eq!(
        AGENT_PROFILES_BINDINGS_DELETE,
        "agentProfiles.bindings.delete"
    );
    assert_eq!(
        AGENT_PROFILES_RETRIEVAL_PREVIEW_CREATE,
        "agentProfiles.retrievalPreview.create"
    );
}
