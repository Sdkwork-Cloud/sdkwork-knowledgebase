use sdkwork_intelligence_knowledgebase_service::knowledge_engine::{
    format_scoped_document_id, parse_namespace_space_id, parse_scoped_document_id,
};

#[test]
fn namespace_and_scoped_document_ids_round_trip() {
    assert_eq!(
        parse_namespace_space_id(Some("space:7")).expect("space id"),
        7
    );
    assert_eq!(
        format_scoped_document_id(7, "concept-ownership"),
        "7/concept-ownership"
    );
    assert_eq!(
        parse_scoped_document_id("7/concept-ownership"),
        Some((7, "concept-ownership".to_string()))
    );
}
