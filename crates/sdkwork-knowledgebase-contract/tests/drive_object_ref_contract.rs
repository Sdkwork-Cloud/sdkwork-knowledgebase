use sdkwork_knowledgebase_contract::KnowledgeDriveObjectRef;

#[test]
fn drive_object_ref_contract_serializes_stable_locator_without_delivery_secrets() {
    let object_ref = KnowledgeDriveObjectRef {
        id: 91,
        space_id: 7,
        drive_space_id: Some("drv-kb-001".to_string()),
        drive_node_id: Some("node-001".to_string()),
        logical_path: Some("raw/documents/report.md".to_string()),
        drive_provider_kind: "sdkwork-drive".to_string(),
        drive_storage_provider_id: "provider-kb".to_string(),
        drive_bucket: "knowledgebase-source".to_string(),
        drive_object_key: "incoming/quarterly-report.md".to_string(),
        drive_object_version: Some("v1".to_string()),
        drive_etag: Some("etag".to_string()),
        content_type: Some("text/markdown; charset=utf-8".to_string()),
        size_bytes: 128,
        checksum_sha256_hex: Some("abc123".to_string()),
        object_role: "original_document".to_string(),
        access_mode: "managed".to_string(),
    };

    let json = serde_json::to_value(object_ref).unwrap();

    assert_eq!(json["spaceId"], 7);
    assert_eq!(json["driveSpaceId"], "drv-kb-001");
    assert_eq!(json["driveNodeId"], "node-001");
    assert_eq!(json["logicalPath"], "raw/documents/report.md");
    assert_eq!(json["driveProviderKind"], "sdkwork-drive");
    assert_eq!(json["driveStorageProviderId"], "provider-kb");
    assert_eq!(json["driveBucket"], "knowledgebase-source");
    assert_eq!(json["driveObjectKey"], "incoming/quarterly-report.md");
    assert_eq!(json["driveObjectVersion"], "v1");
    assert_eq!(json["driveEtag"], "etag");
    assert_eq!(json["objectRole"], "original_document");
    assert_eq!(json["accessMode"], "managed");
    assert!(json.get("presignedUrl").is_none());
    assert!(json.get("providerCredentials").is_none());
    assert!(json.get("payloadBytes").is_none());
}
