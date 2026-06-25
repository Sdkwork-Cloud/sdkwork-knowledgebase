use sdkwork_agent_kernel::{
    KnowledgeDocumentFilter, KnowledgeDocumentKind, KnowledgeProvider, KnowledgeSearchRequest,
};
use sdkwork_knowledgebase_agent_provider::{
    citations_from_okf_concepts, OkfKnowledgeClient, OkfKnowledgeProvider,
    OKF_KNOWLEDGE_PROVIDER_ID,
};
use sdkwork_knowledgebase_contract::okf::OkfConceptSummary;

#[derive(Clone)]
struct FakeOkfClient;

impl OkfKnowledgeClient for FakeOkfClient {
    fn search_okf_concepts(
        &self,
        space_id: u64,
        query: &str,
        top_k: usize,
    ) -> Result<Vec<OkfConceptSummary>, String> {
        assert_eq!(space_id, 7);
        if !query.is_empty() {
            assert_eq!(query, "ownership");
            assert_eq!(top_k, 3);
        }
        Ok(vec![OkfConceptSummary {
            title: "Rust Ownership".to_string(),
            concept_id: "concepts/ownership".to_string(),
            concept_type: "Knowledge Concept".to_string(),
            logical_path: "okf/concepts/ownership.md".to_string(),
            bundle_relative_path: "concepts/ownership.md".to_string(),
            description: "Explains ownership rules".to_string(),
            source_count: 2,
            updated_at: "2026-06-01T00:00:00Z".to_string(),
            tags: vec!["rust".to_string()],
        }])
    }

    fn read_okf_concept_content(
        &self,
        space_id: u64,
        logical_path: &str,
    ) -> Result<String, String> {
        assert_eq!(space_id, 7);
        assert_eq!(logical_path, "okf/concepts/ownership.md");
        Ok("# Ownership".to_string())
    }
}

#[test]
fn okf_provider_search_read_and_list_use_spec_document_ids() {
    let provider = OkfKnowledgeProvider::new(FakeOkfClient);

    let manifest = provider.provider_manifest();
    assert_eq!(manifest.provider_id, OKF_KNOWLEDGE_PROVIDER_ID);

    let results = provider
        .search(
            KnowledgeSearchRequest::new("ownership")
                .with_namespace("space:7")
                .with_top_k(3),
        )
        .expect("search");
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].document_id, "okf:7:concepts/ownership");
    assert_eq!(results[0].document_kind, KnowledgeDocumentKind::Spec);

    let document = provider.read("okf:7:concepts/ownership").expect("read");
    assert_eq!(document.document_id, "okf:7:concepts/ownership");
    assert_eq!(document.content, "# Ownership");

    let listed = provider
        .list(KnowledgeDocumentFilter::new().with_namespace("space:7"))
        .expect("list");
    assert_eq!(listed.len(), 1);
    assert_eq!(listed[0].document_id, "okf:7:concepts/ownership");
}

#[test]
fn okf_citations_use_scoped_logical_path_and_locator() {
    let citations = citations_from_okf_concepts(
        7,
        &[OkfConceptSummary {
            title: "Rust Ownership".to_string(),
            concept_id: "concepts/ownership".to_string(),
            concept_type: "Knowledge Concept".to_string(),
            logical_path: "okf/concepts/ownership.md".to_string(),
            bundle_relative_path: "concepts/ownership.md".to_string(),
            description: "Explains ownership rules".to_string(),
            source_count: 2,
            updated_at: "2026-06-01T00:00:00Z".to_string(),
            tags: vec!["rust".to_string()],
        }],
    );

    assert_eq!(citations.len(), 1);
    assert_eq!(
        citations[0].logical_path.as_deref(),
        Some("7/concepts/ownership")
    );
    assert_eq!(
        citations[0].locator.as_deref(),
        Some("okf:7:concepts/ownership")
    );
    assert_eq!(
        citations[0].source_uri.as_deref(),
        Some("okf/concepts/ownership.md")
    );
}
