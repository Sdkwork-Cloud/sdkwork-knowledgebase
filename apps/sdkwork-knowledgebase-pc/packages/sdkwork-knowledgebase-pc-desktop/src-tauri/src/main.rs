mod desktop_preferences;
mod document_export;
mod document_export_bridge;
mod document_export_html;
mod document_export_webview;
mod export_save;
mod resource_bridge;
mod tray_bridge;

use std::sync::atomic::Ordering;

use desktop_preferences::{
    get_autostart_enabled, get_desktop_host_status, handle_close_requested, sync_desktop_preferences,
    DesktopPreferenceState,
};
use document_export_bridge::export_document_pdf;
use export_save::{locate_export_file, open_export_file, reveal_export_file, save_export_file};
use resource_bridge::{
    fetch_binary_resource, open_external_url, read_local_resource, save_binary_resource,
};
use serde::Deserialize;
use tauri::{AppHandle, Manager, WindowEvent};
use tray_bridge::{install_system_tray, sync_tray_locale};

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
        .manage(DesktopPreferenceState::new(true))
        .setup(|app| {
            let tray_state = install_system_tray(app.handle())?;
            app.manage(tray_state);

            let app_handle = app.handle().clone();
            let hide_to_tray = app.state::<DesktopPreferenceState>().hide_to_tray.clone();

            if let Some(window) = app.get_webview_window("main") {
                window.on_window_event(move |event| {
                    if let WindowEvent::CloseRequested { api, .. } = event {
                        let should_hide = hide_to_tray.load(Ordering::Relaxed);
                        if should_hide {
                            api.prevent_close();
                            handle_close_requested(&app_handle, &hide_to_tray);
                        }
                    }
                });
            }

            Ok(())
        })
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
            export_document_pdf,
            sync_desktop_preferences,
            get_desktop_host_status,
            get_autostart_enabled,
            sync_tray_locale
        ])
        .run(tauri::generate_context!())
        .expect("failed to run SDKWork Knowledgebase desktop host");
}
