use sdkwork_intelligence_knowledgebase_service::ports::knowledge_drive_storage::{
    HeadKnowledgeObjectRequest, KnowledgeDriveStorage, KnowledgeObjectRef, KnowledgeStorageError,
    PutKnowledgeObjectRequest,
};

fn assert_storage_port_is_object_safe(_: &dyn KnowledgeDriveStorage) {}

#[test]
fn service_exposes_drive_storage_port() {
    struct Placeholder;

    #[async_trait::async_trait]
    impl KnowledgeDriveStorage for Placeholder {
        async fn put_object(
            &self,
            _request: PutKnowledgeObjectRequest,
        ) -> Result<KnowledgeObjectRef, KnowledgeStorageError> {
            Err(KnowledgeStorageError::internal("not implemented"))
        }

        async fn head_object(
            &self,
            _request: HeadKnowledgeObjectRequest,
        ) -> Result<KnowledgeObjectRef, KnowledgeStorageError> {
            Err(KnowledgeStorageError::internal("not implemented"))
        }

        async fn get_object_text(
            &self,
            _object_ref: &KnowledgeObjectRef,
        ) -> Result<String, KnowledgeStorageError> {
            Err(KnowledgeStorageError::internal("not implemented"))
        }
    }

    assert_storage_port_is_object_safe(&Placeholder);
}

#[test]
fn storage_requests_preserve_logical_path_and_role() {
    let request = PutKnowledgeObjectRequest::text(
        "wiki/index.md",
        "wiki_index",
        "# Index",
        Some("abc123".to_string()),
    );

    assert_eq!(request.logical_path, "wiki/index.md");
    assert_eq!(request.object_role, "wiki_index");
    assert_eq!(request.content_type, "text/markdown; charset=utf-8");
    assert_eq!(request.body, b"# Index");
    assert_eq!(request.checksum_sha256_hex.as_deref(), Some("abc123"));
}
