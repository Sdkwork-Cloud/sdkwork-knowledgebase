use sdkwork_knowledgebase_test_support::fake_drive::FakeKnowledgeDriveStorage;

#[tokio::test]
async fn fake_drive_puts_and_reads_text_by_object_ref() {
    let drive = FakeKnowledgeDriveStorage::default();

    let object_ref = drive
        .put_text("wiki/index.md", "wiki_index", "# Index")
        .await
        .unwrap();

    assert_eq!(object_ref.logical_path, "wiki/index.md");
    assert_eq!(object_ref.object_role, "wiki_index");
    assert_eq!(drive.read_text(&object_ref).await.unwrap(), "# Index");
    assert!(object_ref.checksum_sha256_hex.is_some());
}

#[tokio::test]
async fn fake_drive_rejects_missing_objects() {
    let drive = FakeKnowledgeDriveStorage::default();
    let object_ref = drive
        .put_text("wiki/log.md", "wiki_log", "# Log")
        .await
        .unwrap();

    drive.clear().await;

    assert!(drive.read_text(&object_ref).await.is_err());
}
