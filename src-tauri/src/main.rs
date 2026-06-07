// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod clipboard;
mod db;

use anyhow::Result;
use db::Database;
use std::sync::atomic::AtomicBool;
use tauri::{Builder, Manager};

#[derive(Debug)]
pub struct AppState {
    pub db: Database,
    pub skip_event: AtomicBool,
}

async fn run_tauri_app() -> Result<()> {
    let state = AppState {
        db: Database::new().await?,
        skip_event: AtomicBool::new(false),
    };

    Builder::default()
        .setup(|app| {
            app.manage(state);
            clipboard::start_clipboard_listener(app.handle().clone())?;
            Ok(())
        })
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_prevent_default::init())
        .invoke_handler(tauri::generate_handler![
            clipboard::copy_item,
            clipboard::paste_item,
            clipboard::clear_history,
        ])
        .run(tauri::generate_context!())?;

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    run_tauri_app().await?;
    Ok(())
}
