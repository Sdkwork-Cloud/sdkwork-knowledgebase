use sdkwork_knowledgebase_contract::ingest::{KnowledgeDriveImportRequest, KnowledgeIngestRequest};

#[test]
fn ingest_request_serializes_markdown_payload_without_storage_locator_leaks() {
    let request = KnowledgeIngestRequest {
        space_id: 7,
        title: "API payload note".to_string(),
        payload_markdown: "# API Note\n\nImportant source text.".to_string(),
        idempotency_key: "api-note-1".to_string(),
    };

    let json = serde_json::to_value(request).unwrap();

    assert_eq!(json["spaceId"], 7);
    assert_eq!(
        json["payloadMarkdown"],
        "# API Note\n\nImportant source text."
    );
    assert!(json.get("driveObjectKey").is_none());
    assert!(json.get("presignedUrl").is_none());
}

#[test]
fn drive_import_request_uses_stable_drive_locator_and_idempotency_key() {
    let request = KnowledgeDriveImportRequest {
        space_id: 7,
        title: "Quarterly Report".to_string(),
        drive_bucket: "knowledgebase-source".to_string(),
        drive_object_key: "incoming/quarterly-report.md".to_string(),
        idempotency_key: "drive-quarterly-report".to_string(),
        language: Some("en".to_string()),
    };

    let json = serde_json::to_value(request).unwrap();

    assert_eq!(json["spaceId"], 7);
    assert_eq!(json["driveBucket"], "knowledgebase-source");
    assert_eq!(json["driveObjectKey"], "incoming/quarterly-report.md");
    assert_eq!(json["idempotencyKey"], "drive-quarterly-report");
    assert!(json.get("fileBytes").is_none());
}
