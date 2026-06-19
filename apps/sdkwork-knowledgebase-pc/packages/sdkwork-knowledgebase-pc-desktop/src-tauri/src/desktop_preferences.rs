use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager, State};

const AUTOSTART_REGISTRY_VALUE: &str = "SDKWork Knowledgebase";

#[cfg(target_os = "macos")]
const MACOS_LAUNCH_AGENT_LABEL: &str = "com.sdkwork.sdkwork.knowledgebase.desktop";

#[cfg(target_os = "linux")]
const LINUX_AUTOSTART_FILE_NAME: &str = "com.sdkwork.sdkwork.knowledgebase.desktop.desktop";

pub struct DesktopPreferenceState {
    pub hide_to_tray: Arc<AtomicBool>,
}

impl DesktopPreferenceState {
    pub fn new(hide_to_tray: bool) -> Self {
        Self {
            hide_to_tray: Arc::new(AtomicBool::new(hide_to_tray)),
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncDesktopPreferencesRequest {
    hide_to_tray: bool,
    auto_start: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AutostartStatusResponse {
    enabled: bool,
    supported: bool,
    platform: String,
    hide_to_tray_supported: bool,
}

#[tauri::command]
pub fn sync_desktop_preferences(
    state: State<'_, DesktopPreferenceState>,
    request: SyncDesktopPreferencesRequest,
) -> Result<(), String> {
    state
        .hide_to_tray
        .store(request.hide_to_tray, Ordering::Relaxed);
    set_autostart_enabled(request.auto_start)?;
    Ok(())
}

#[tauri::command]
pub fn get_desktop_host_status() -> Result<AutostartStatusResponse, String> {
    Ok(AutostartStatusResponse {
        enabled: read_autostart_enabled()?,
        supported: autostart_supported(),
        platform: current_platform_label(),
        hide_to_tray_supported: hide_to_tray_supported(),
    })
}

#[tauri::command]
pub fn get_autostart_enabled() -> Result<AutostartStatusResponse, String> {
    get_desktop_host_status()
}

pub fn current_platform_label() -> String {
    if cfg!(windows) {
        "windows".to_string()
    } else if cfg!(target_os = "macos") {
        "macos".to_string()
    } else if cfg!(target_os = "linux") {
        "linux".to_string()
    } else {
        "unknown".to_string()
    }
}

pub fn autostart_supported() -> bool {
    cfg!(any(windows, target_os = "macos", target_os = "linux"))
}

pub fn hide_to_tray_supported() -> bool {
    cfg!(any(windows, target_os = "macos", target_os = "linux"))
}

pub fn read_autostart_enabled() -> Result<bool, String> {
    #[cfg(windows)]
    {
        use winreg::enums::HKEY_CURRENT_USER;
        use winreg::RegKey;

        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let run = hkcu
            .open_subkey("Software\\Microsoft\\Windows\\CurrentVersion\\Run")
            .map_err(|error| error.to_string())?;

        let stored: String = run
            .get_value(AUTOSTART_REGISTRY_VALUE)
            .unwrap_or_default();
        if stored.is_empty() {
            return Ok(false);
        }

        let current_exe = std::env::current_exe().map_err(|error| error.to_string())?;
        let stored_path = PathBuf::from(stored.trim_matches('"'));
        return Ok(paths_equivalent(&stored_path, &current_exe));
    }

    #[cfg(target_os = "macos")]
    {
        return macos_autostart_matches_current_exe();
    }

    #[cfg(target_os = "linux")]
    {
        return linux_autostart_matches_current_exe();
    }

    #[cfg(not(any(windows, target_os = "macos", target_os = "linux")))]
    {
        Ok(false)
    }
}

pub fn set_autostart_enabled(enabled: bool) -> Result<(), String> {
    #[cfg(windows)]
    {
        use winreg::enums::{HKEY_CURRENT_USER, KEY_SET_VALUE, KEY_WRITE};
        use winreg::RegKey;

        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let run = hkcu
            .open_subkey_with_flags(
                "Software\\Microsoft\\Windows\\CurrentVersion\\Run",
                KEY_WRITE | KEY_SET_VALUE,
            )
            .map_err(|error| error.to_string())?;

        if enabled {
            let exe = std::env::current_exe().map_err(|error| error.to_string())?;
            run.set_value(
                AUTOSTART_REGISTRY_VALUE,
                &format!("\"{}\"", exe.to_string_lossy()),
            )
            .map_err(|error| error.to_string())?;
        } else {
            let _ = run.delete_value(AUTOSTART_REGISTRY_VALUE);
        }

        return Ok(());
    }

    #[cfg(target_os = "macos")]
    {
        return set_macos_autostart_enabled(enabled);
    }

    #[cfg(target_os = "linux")]
    {
        return set_linux_autostart_enabled(enabled);
    }

    #[cfg(not(any(windows, target_os = "macos", target_os = "linux")))]
    {
        let _ = enabled;
        Err("autostart is not supported on this platform".to_string())
    }
}

fn paths_equivalent(left: &Path, right: &Path) -> bool {
    if left == right {
        return true;
    }

    match (std::fs::canonicalize(left), std::fs::canonicalize(right)) {
        (Ok(left_path), Ok(right_path)) => left_path == right_path,
        _ => false,
    }
}

#[cfg(target_os = "linux")]
fn quote_executable_path(path: &Path) -> String {
    format!("\"{}\"", path.to_string_lossy().replace('"', "\\\""))
}

#[cfg(target_os = "macos")]
fn macos_launch_agent_path() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("Library/LaunchAgents")
        .join(format!("{MACOS_LAUNCH_AGENT_LABEL}.plist"))
}

#[cfg(target_os = "macos")]
fn macos_autostart_matches_current_exe() -> Result<bool, String> {
    let plist_path = macos_launch_agent_path();
    if !plist_path.is_file() {
        return Ok(false);
    }

    let content = std::fs::read_to_string(&plist_path).map_err(|error| error.to_string())?;
    let current_exe = std::env::current_exe().map_err(|error| error.to_string())?;
    Ok(content.contains(&current_exe.to_string_lossy().to_string()))
}

#[cfg(target_os = "macos")]
fn macos_gui_domain() -> String {
    let uid = std::process::Command::new("id")
        .arg("-u")
        .output()
        .ok()
        .and_then(|output| String::from_utf8(output.stdout).ok())
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "501".to_string());

    format!("gui/{uid}")
}

#[cfg(target_os = "macos")]
fn set_macos_autostart_enabled(enabled: bool) -> Result<(), String> {
    let plist_path = macos_launch_agent_path();
    let gui_domain = macos_gui_domain();

    if enabled {
        let exe = std::env::current_exe().map_err(|error| error.to_string())?;
        if let Some(parent) = plist_path.parent() {
            std::fs::create_dir_all(parent).map_err(|error| error.to_string())?;
        }

        let plist = format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>{MACOS_LAUNCH_AGENT_LABEL}</string>
    <key>ProgramArguments</key>
    <array>
        <string>{}</string>
    </array>
    <key>RunAtLoad</key>
    <true/>
    <key>KeepAlive</key>
    <false/>
</dict>
</plist>"#,
            exe.display()
        );
        std::fs::write(&plist_path, plist).map_err(|error| error.to_string())?;

        let plist_arg = plist_path.to_string_lossy().to_string();
        let status = std::process::Command::new("launchctl")
            .args(["bootstrap", &gui_domain, &plist_arg])
            .status()
            .or_else(|_| {
                std::process::Command::new("launchctl")
                    .args(["load", "-w", &plist_arg])
                    .status()
            })
            .map_err(|error| error.to_string())?;

        if !status.success() {
            return Err("failed to register macOS launch agent".to_string());
        }
    } else {
        let plist_arg = plist_path.to_string_lossy().to_string();
        let _ = std::process::Command::new("launchctl")
            .args(["bootout", &gui_domain, &plist_arg])
            .status()
            .or_else(|_| {
                std::process::Command::new("launchctl")
                    .args(["unload", "-w", &plist_arg])
                    .status()
            });

        let _ = std::fs::remove_file(plist_path);
    }

    Ok(())
}

#[cfg(target_os = "linux")]
fn linux_autostart_desktop_path() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("autostart")
        .join(LINUX_AUTOSTART_FILE_NAME)
}

#[cfg(target_os = "linux")]
fn linux_read_exec_from_desktop(content: &str) -> Option<String> {
    content
        .lines()
        .map(str::trim)
        .find(|line| line.starts_with("Exec="))
        .map(|line| line.trim_start_matches("Exec=").trim().to_string())
}

#[cfg(target_os = "linux")]
fn linux_autostart_matches_current_exe() -> Result<bool, String> {
    let desktop_path = linux_autostart_desktop_path();
    if !desktop_path.is_file() {
        return Ok(false);
    }

    let content = std::fs::read_to_string(&desktop_path).map_err(|error| error.to_string())?;
    let Some(stored_exec) = linux_read_exec_from_desktop(&content) else {
        return Ok(false);
    };

    let current_exe = std::env::current_exe().map_err(|error| error.to_string())?;
    let stored_path = PathBuf::from(
        stored_exec
            .trim_matches('"')
            .split_whitespace()
            .next()
            .unwrap_or(&stored_exec),
    );

    Ok(paths_equivalent(&stored_path, &current_exe))
}

#[cfg(target_os = "linux")]
fn set_linux_autostart_enabled(enabled: bool) -> Result<(), String> {
    let desktop_path = linux_autostart_desktop_path();

    if enabled {
        let exe = std::env::current_exe().map_err(|error| error.to_string())?;
        if let Some(parent) = desktop_path.parent() {
            std::fs::create_dir_all(parent).map_err(|error| error.to_string())?;
        }

        let desktop_entry = format!(
            "[Desktop Entry]\n\
             Type=Application\n\
             Version=1.0\n\
             Name=SDKWork Knowledgebase\n\
             Comment=SDKWork Knowledgebase desktop client\n\
             Exec={}\n\
             Terminal=false\n\
             Categories=Office;Utility;\n\
             Hidden=false\n\
             StartupNotify=false\n\
             StartupWMClass=com.sdkwork.sdkwork.knowledgebase.desktop\n\
             X-GNOME-Autostart-enabled=true\n\
             X-GNOME-Autostart-Delay=0\n\
             X-KDE-autostart-after=panel\n\
             X-KDE-StartupNotify=false\n",
            quote_executable_path(&exe)
        );

        std::fs::write(&desktop_path, desktop_entry).map_err(|error| error.to_string())?;
    } else if desktop_path.exists() {
        std::fs::remove_file(desktop_path).map_err(|error| error.to_string())?;
    }

    Ok(())
}

pub fn handle_close_requested(app: &AppHandle, hide_to_tray: &Arc<AtomicBool>) {
    if !hide_to_tray.load(Ordering::Relaxed) {
        return;
    }

    if let Some(window) = app.get_webview_window("main") {
        let _ = window.hide();
    }
}
