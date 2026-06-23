// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod clipboard;
mod db;

use anyhow::Result;
use db::Database;
use std::fs::create_dir_all;
use std::sync::atomic::AtomicBool;
use tauri::{
    image::Image,
    menu::{Menu, MenuItem},
    tray::TrayIconBuilder,
    Builder, Manager,
};

async fn run_tauri_app() -> Result<()> {
    let mut db_url = dirs::data_dir().expect("Failed to get data directory");
    db_url.push("Squirrel");
    if !db_url.exists() {
        create_dir_all(&db_url)?;
    }
    db_url.push("data.db");
    let db = Database::new(db_url.to_str().unwrap()).await?;

    // Whether or not to ignore clipboard events
    let skip_event = AtomicBool::new(false);

    Builder::default()
        .setup(move |app| {
            let icon_bytes = include_bytes!("../icons/64x64.png");
            let icon = Image::from_bytes(icon_bytes).expect("Failed to parse icon bytes");

            let quit_i = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let show_i = MenuItem::with_id(app, "show", "Show window", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show_i, &quit_i])?;

            TrayIconBuilder::new()
                .menu(&menu)
                .icon(icon)
                .build(app)?
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "quit" => {
                        app.exit(0);
                    }
                    "show" => {
                        let _ = app.get_webview_window("main").unwrap().set_focus();
                        let _ = app.get_webview_window("main").unwrap().show();
                    }
                    _ => {
                        println!("Unhandled menu item: {:?}", event.id);
                    }
                });

            app.manage(db);
            app.manage(skip_event);
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
            clipboard::remove_entry,
            clipboard::load_history,
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
