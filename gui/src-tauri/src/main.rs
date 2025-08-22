// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use ledger_module::{init_db, open_db, list_entries, add_entry, Kind};
use tauri::{Manager, WindowEvent};

#[tauri::command]
fn list() -> Result<Vec<serde_json::Value>, String> {
    let conn = open_db().map_err(|e| e.to_string())?;
    init_db(&conn).map_err(|e| e.to_string())?;
    let entries = list_entries(&conn).map_err(|e| e.to_string())?;

    Ok(entries.into_iter().map(|e| {
        serde_json::json!({
            "id": e.id,
            "kind": if e.kind == Kind::Expense { "expense" } else { "income" },
            "amount": e.amount,
            "category": e.category,
            "note": e.note,
            "created_at": e.created_at,
        })
    })
    .collect())
}

#[tauri::command]
fn add(kind: String, amount: i64, category: String, note: Option<String>) -> Result<(), String> {
    let kind = match kind.as_str() {
        "expense" => Kind::Expense,
        "income" => Kind::Income,
        _ => return Err("kind must be 'expense' or 'income'".into()),
    };
    let conn = open_db().map_err(|e| e.to_string())?;
    init_db(&conn).map_err(|e| e.to_string())?;
    add_entry(&conn, kind, amount, &category, note.as_deref()).map_err(|e| e.to_string())
}

fn main() {
  tauri::Builder::default()
    .setup(|app| {
      let dir = app.path().app_data_dir()?;
      std::fs::create_dir_all(&dir)?;
      std::env::set_current_dir(&dir)?;
      Ok(())
    })
    .on_window_event(|w, e| {
      #[cfg(debug_assertions)]
      if matches!(e, WindowEvent::Focused(true)) && w.label() == "main" {
        // Window -> WebviewWindow に取り直してから devtools を開く
        if let Some(webview) = w.app_handle().get_webview_window(w.label()) {
          webview.open_devtools();
          // webview.set_focus().ok(); // 任意
        }
      }
    })
    .invoke_handler(tauri::generate_handler![list, add])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
