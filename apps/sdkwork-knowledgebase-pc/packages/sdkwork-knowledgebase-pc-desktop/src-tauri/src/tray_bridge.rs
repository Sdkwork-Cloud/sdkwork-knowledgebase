use serde::Deserialize;
use tauri::{
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::{MouseButton, MouseButtonState, TrayIcon, TrayIconBuilder, TrayIconEvent},
    AppHandle, Emitter, Manager, State,
};

pub const TRAY_ICON_ID: &str = "sdkwork-knowledgebase-main-tray";
pub const OPEN_SETTINGS_EVENT: &str = "open-settings";

pub struct TrayMenuState {
    pub show_item: MenuItem<tauri::Wry>,
    pub settings_item: MenuItem<tauri::Wry>,
    pub quit_item: MenuItem<tauri::Wry>,
    pub tray_icon: TrayIcon,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncTrayLocaleRequest {
    show_label: String,
    settings_label: String,
    quit_label: String,
    tooltip: String,
}

pub fn install_system_tray(app: &AppHandle) -> Result<TrayMenuState, Box<dyn std::error::Error>> {
    let show_item = MenuItem::with_id(app, "tray-show", "Show Window", true, None::<&str>)?;
    let settings_item = MenuItem::with_id(app, "tray-settings", "Settings", true, None::<&str>)?;
    let quit_item = MenuItem::with_id(app, "tray-quit", "Quit", true, None::<&str>)?;
    let separator = PredefinedMenuItem::separator(app)?;
    let menu = Menu::with_items(
        app,
        &[
            &show_item,
            &settings_item,
            &separator,
            &quit_item,
        ],
    )?;

    let tray_icon = TrayIconBuilder::with_id(TRAY_ICON_ID)
        .icon(
            app.default_window_icon()
                .ok_or("default window icon is unavailable")?
                .clone(),
        )
        .tooltip("SDKWork Knowledgebase")
        .menu(&menu)
        .on_menu_event(|app, event| match event.id.as_ref() {
            "tray-show" => show_main_window(app),
            "tray-settings" => open_settings_from_tray(app),
            "tray-quit" => {
                app.exit(0);
            }
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                show_main_window(tray.app_handle());
            }
        })
        .build(app)?;

    Ok(TrayMenuState {
        show_item,
        settings_item,
        quit_item,
        tray_icon,
    })
}

#[tauri::command]
pub fn sync_tray_locale(
    state: State<'_, TrayMenuState>,
    request: SyncTrayLocaleRequest,
) -> Result<(), String> {
    state
        .show_item
        .set_text(request.show_label.trim())
        .map_err(|error| error.to_string())?;
    state
        .settings_item
        .set_text(request.settings_label.trim())
        .map_err(|error| error.to_string())?;
    state
        .quit_item
        .set_text(request.quit_label.trim())
        .map_err(|error| error.to_string())?;
    state
        .tray_icon
        .set_tooltip(Some(request.tooltip.trim()))
        .map_err(|error| error.to_string())?;
    Ok(())
}

fn show_main_window(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.unminimize();
        let _ = window.set_focus();
    }
}

fn open_settings_from_tray(app: &AppHandle) {
    show_main_window(app);
    let _ = app.emit(OPEN_SETTINGS_EVENT, ());
}
