#[cfg(feature = "gui")]
mod ollama;

#[cfg(feature = "gui")]
use ollama::{OllamaClient, TranslateRequest, TranslateResponse};
#[cfg(feature = "gui")]
use tauri::{State, Manager, AppHandle, Emitter};
#[cfg(feature = "gui")]
use std::sync::Arc;
#[cfg(feature = "gui")]
use tokio::sync::Mutex;
#[cfg(feature = "gui")]
use tauri_plugin_clipboard_manager::ClipboardExt;
#[cfg(feature = "gui")]
use tauri_plugin_global_shortcut::GlobalShortcutExt;

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[cfg(feature = "gui")]
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg(feature = "gui")]
#[tauri::command]
async fn translate(
    text: String,
    from_lang: String,
    to_lang: String,
    state: State<'_, Arc<Mutex<OllamaClient>>>,
) -> Result<TranslateResponse, String> {
    let client = state.lock().await;
    let request = TranslateRequest {
        text,
        from_lang,
        to_lang,
    };
    client.translate(request).await
}

#[cfg(feature = "gui")]
#[tauri::command]
async fn check_ollama_health(
    state: State<'_, Arc<Mutex<OllamaClient>>>,
) -> Result<bool, String> {
    let client = state.lock().await;
    client.check_health().await
}

#[cfg(feature = "gui")]
#[tauri::command]
async fn get_clipboard_text(app: AppHandle) -> Result<String, String> {
    match app.clipboard().read_text() {
        Ok(text) => Ok(text),
        Err(e) => Err(format!("Failed to read clipboard: {}", e)),
    }
}

#[cfg(feature = "gui")]
#[tauri::command]
async fn set_clipboard_text(app: AppHandle, text: String) -> Result<(), String> {
    app.clipboard()
        .write_text(text)
        .map_err(|e| format!("Failed to write to clipboard: {}", e))
}

#[cfg(feature = "gui")]
#[tauri::command]
fn show_window(window: tauri::Window) -> Result<(), String> {
    window.show().map_err(|e| e.to_string())?;
    window.set_focus().map_err(|e| e.to_string())?;
    Ok(())
}

#[cfg(feature = "gui")]
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let ollama_client = Arc::new(Mutex::new(OllamaClient::new()));

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .manage(ollama_client)
        .invoke_handler(tauri::generate_handler![
            greet,
            translate,
            check_ollama_health,
            get_clipboard_text,
            set_clipboard_text,
            show_window
        ])
        .setup(|app| {
            let app_handle = app.handle().clone();
            
            // Register global shortcut for translation (Cmd+Shift+T on macOS)
            app.global_shortcut().on_shortcut("CmdOrCtrl+Shift+T", move |_app, _shortcut, _event| {
                if let Some(window) = app_handle.get_webview_window("main") {
                    let _ = window.show();
                    let _ = window.set_focus();
                    // Emit event to frontend to trigger clipboard read
                    let _ = window.emit("translate-shortcut", ());
                }
            })?;
            
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
