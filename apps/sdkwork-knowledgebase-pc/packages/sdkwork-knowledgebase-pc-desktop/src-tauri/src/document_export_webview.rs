use crate::document_export_html::build_html_document;
use base64::{engine::general_purpose::STANDARD, Engine as _};
use std::sync::mpsc;
use tauri::{AppHandle, Url, WebviewUrl, WebviewWindowBuilder};

#[cfg(windows)]
pub async fn export_html_to_pdf(app: &AppHandle, title: &str, body_html: &str) -> Result<Vec<u8>, String> {
    let app = app.clone();
    let title = title.to_string();
    let body_html = body_html.to_string();

    tauri::async_runtime::spawn_blocking(move || {
        let (tx, rx) = mpsc::sync_channel(1);
        let app_for_thread = app.clone();
        app.run_on_main_thread(move || {
            let _ = tx.send(export_html_to_pdf_on_main_thread(&app_for_thread, &title, &body_html));
        })
        .map_err(|error| format!("failed to dispatch HTML PDF export: {error}"))?;

        rx.recv()
            .map_err(|error| format!("HTML PDF export result channel failed: {error}"))?
    })
    .await
    .map_err(|error| format!("HTML PDF export task failed: {error}"))?
}

#[cfg(not(windows))]
pub async fn export_html_to_pdf(
    _app: &AppHandle,
    _title: &str,
    _body_html: &str,
) -> Result<Vec<u8>, String> {
    Err("HTML WebView PDF export is only available on Windows".to_string())
}

#[cfg(windows)]
fn export_html_to_pdf_on_main_thread(
    app: &AppHandle,
    title: &str,
    body_html: &str,
) -> Result<Vec<u8>, String> {
    use std::thread;
    use std::time::Duration;
    use webview2_com::Microsoft::Web::WebView2::Win32::ICoreWebView2_7;
    use webview2_com::PrintToPdfCompletedHandler;
    use windows::core::{Interface, HSTRING};

    let html = build_html_document(title, body_html);
    let data_url = format!(
        "data:text/html;charset=utf-8;base64,{}",
        STANDARD.encode(html.as_bytes())
    );
    let url = Url::parse(&data_url).map_err(|error| format!("invalid export URL: {error}"))?;

    let label = format!(
        "pdf-export-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|duration| duration.as_nanos())
            .unwrap_or(0)
    );
    let pdf_path = std::env::temp_dir().join(format!("{label}.pdf"));
    if pdf_path.exists() {
        std::fs::remove_file(&pdf_path).map_err(|error| format!("temp pdf cleanup failed: {error}"))?;
    }

    let window = WebviewWindowBuilder::new(app, &label, WebviewUrl::External(url))
        .visible(false)
        .focused(false)
        .skip_taskbar(true)
        .inner_size(820.0, 1169.0)
        .build()
        .map_err(|error| format!("failed to create export webview: {error}"))?;

    let export_result = (|| -> Result<Vec<u8>, String> {
        thread::sleep(Duration::from_millis(1600));

        let pdf_path_string = pdf_path.to_string_lossy().into_owned();
        let (tx, rx) = mpsc::sync_channel(1);
        window
            .with_webview(move |platform| {
                let result = (|| -> Result<(), String> {
                    unsafe {
                        let core = platform
                            .controller()
                            .CoreWebView2()
                            .map_err(|error| format!("CoreWebView2 unavailable: {error}"))?;
                        let core = core
                            .cast::<ICoreWebView2_7>()
                            .map_err(|_| "PrintToPdf requires WebView2 ICoreWebView2_7".to_string())?;

                        PrintToPdfCompletedHandler::wait_for_async_operation(
                            Box::new({
                                let pdf_path_string = pdf_path_string.clone();
                                move |handler| {
                                    core.PrintToPdf(&HSTRING::from(&pdf_path_string), None, &handler)
                                        .map_err(webview2_com::Error::WindowsError)
                                }
                            }),
                            Box::new(|result, success| {
                                result?;
                                if success {
                                    Ok(())
                                } else {
                                    Err(windows::core::Error::from_hresult(windows::core::HRESULT(-1)))
                                }
                            }),
                        )
                        .map_err(|error| format!("PrintToPdf failed: {error:?}"))?;
                    }
                    Ok(())
                })();

                let _ = tx.send(result);
            })
            .map_err(|error| format!("export webview unavailable: {error}"))?;

        rx.recv()
            .map_err(|error| format!("PrintToPdf result channel failed: {error}"))??;

        let bytes =
            std::fs::read(&pdf_path).map_err(|error| format!("failed to read exported pdf: {error}"))?;

        if bytes.starts_with(b"%PDF") {
            Ok(bytes)
        } else {
            Err("exported file is not a valid PDF".to_string())
        }
    })();

    window.close().ok();
    std::fs::remove_file(&pdf_path).ok();
    export_result
}
