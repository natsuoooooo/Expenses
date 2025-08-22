// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use ledger_module::{add_entry, category_totals_by_kind, delete_entry, init_db, list_entries, open_db, month_summary, Kind};
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

#[tauri::command]
fn delete(id: i64) -> Result<bool, String> {
    let conn = open_db().map_err(|e| e.to_string())?;
    init_db(&conn).map_err(|e| e.to_string())?;
    let affected = delete_entry(&conn, id).map_err(|e| e.to_string())?;
    Ok(affected > 0)
}

#[tauri::command]
fn get_month_summary(ym: String) -> Result<serde_json::Value, String> {
    let conn = open_db().map_err(|e| e.to_string())?;
    init_db(&conn).map_err(|e| e.to_string())?;
    let s = month_summary(&conn, &ym).map_err(|e| e.to_string())?;
    Ok(serde_json::json!({
        "month": s.month,
        "income": s.income,
        "expense": s.expense,
        "balance": s.balance
    }))
}

#[tauri::command]
fn get_category_totals(ym: String, kind: String) -> Result<Vec<serde_json::Value>, String> {
    let k = match kind.as_str() {
        "expense" => Kind::Expense,
        "income" => Kind::Income,
        _ => return Err("kind must be 'expense' or 'income'".into()),
    };
    let conn = open_db().map_err(|e| e.to_string())?;
    init_db(&conn).map_err(|e| e.to_string())?;
    let rows = category_totals_by_kind(&conn, &ym, k).map_err(|e| e.to_string())?;
    Ok(rows.into_iter().map(|t| {
        serde_json::json!({
            "category": t.category,
            "total": t.total,
        })
    }).collect())
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
    .invoke_handler(tauri::generate_handler![list, add, delete, get_month_summary, get_category_totals])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
