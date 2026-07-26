#[test]
fn embedded_iam_preserves_router_manifest_framework_binding() {
    let source = include_str!("../src/bootstrap.rs");

    assert!(source.contains("bootstrap_iam_for_application"));
    assert!(source.contains("wrap_router_with_iam_owner_web_framework"));
    assert!(!source.contains("assemble_api_router().await.router"));
}
