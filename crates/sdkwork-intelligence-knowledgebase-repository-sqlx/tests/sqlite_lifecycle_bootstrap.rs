use sdkwork_intelligence_knowledgebase_repository_sqlx::connect_knowledgebase_and_install_schema;

#[tokio::test]
async fn authoritative_repository_rejects_file_backed_sqlite() {
    let error = connect_knowledgebase_and_install_schema(
        "sqlite://target/repository-sqlite-tests/forbidden.db?mode=rwc",
    )
    .await
    .expect_err("authoritative Knowledgebase persistence must reject file-backed SQLite");
    assert!(error
        .to_string()
        .contains("client-local SQLite must be owned by a native/client persistence module"));
}
