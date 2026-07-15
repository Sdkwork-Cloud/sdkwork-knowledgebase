fn main() {
    let attributes =
        tauri_build::Attributes::new().app_manifest(tauri_build::AppManifest::new().commands(&[
            "window_control",
            "fetch_binary_resource",
            "read_local_resource",
            "open_external_url",
            "save_binary_resource",
            "save_export_file",
            "reveal_export_file",
            "open_export_file",
            "locate_export_file",
            "export_document_pdf",
            "write_secure_session_value",
            "remove_secure_session_value",
            "clear_secure_session_values",
            "read_secure_session_snapshot",
            "take_pending_group_knowledgebase_launch",
        ]));

    tauri_build::try_build(attributes)
        .expect("failed to run SDKWork Knowledgebase desktop build script");
}
