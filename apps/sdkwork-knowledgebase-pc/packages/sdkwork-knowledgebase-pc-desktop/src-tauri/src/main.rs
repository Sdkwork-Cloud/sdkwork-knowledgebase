mod document_export;
mod document_export_bridge;
mod document_export_html;
mod document_export_webview;
mod export_save;
mod resource_bridge;

use document_export_bridge::export_document_pdf;
use export_save::{locate_export_file, open_export_file, reveal_export_file, save_export_file};
use resource_bridge::{
    fetch_binary_resource, open_external_url, read_local_resource, save_binary_resource,
};
use serde::Deserialize;
use tauri::{AppHandle, Manager};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct WindowControlRequest {
    action: WindowControlAction,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
enum WindowControlAction {
    Minimize,
    Maximize,
    Unmaximize,
    Close,
    Show,
}

#[tauri::command]
fn window_control(app: AppHandle, request: WindowControlRequest) -> Result<(), String> {
    let window = app
        .get_webview_window("main")
        .ok_or_else(|| "main window is unavailable".to_string())?;

    match request.action {
        WindowControlAction::Minimize => window.minimize(),
        WindowControlAction::Maximize => window.maximize(),
        WindowControlAction::Unmaximize => window.unmaximize(),
        WindowControlAction::Close => window.close(),
        WindowControlAction::Show => window.show(),
    }
    .map_err(|_| "window control failed".to_string())
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            window_control,
            fetch_binary_resource,
            read_local_resource,
            open_external_url,
            save_binary_resource,
            save_export_file,
            reveal_export_file,
            open_export_file,
            locate_export_file,
            export_document_pdf
        ])
        .run(tauri::generate_context!())
        .expect("failed to run SDKWork Knowledgebase desktop host");
}
