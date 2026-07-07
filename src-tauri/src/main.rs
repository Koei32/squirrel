// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod clipboard;
mod commands;
mod db;

use anyhow::Result;
use db::Database;
use std::sync::atomic::AtomicBool;
use std::{ffi::OsStr, fs::create_dir_all};
use sysinfo::{ProcessRefreshKind, RefreshKind};
use tauri::{
    image::Image,
    menu::{Menu, MenuItem},
    tray::TrayIconBuilder,
    Builder, Manager,
};

async fn run_tauri_app(silent: bool) -> Result<()> {
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
            let window = app
                .get_webview_window("main")
                .expect("Failed to get main webview window");

            if !silent {
                window.show()?;
            }

            // Tray icon setup
            let icon_bytes = include_bytes!("../icons/64x64.png");
            let icon = Image::from_bytes(icon_bytes).expect("Failed to parse icon bytes");
            let quit_i = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let show_i = MenuItem::with_id(app, "show", "Show window", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show_i, &quit_i])?;
            TrayIconBuilder::new()
                .menu(&menu)
                .icon(icon)
                .build(app)?
                .on_menu_event(move |app, event| match event.id.as_ref() {
                    "quit" => {
                        app.exit(0);
                    }
                    "show" => {
                        let _ = window.set_focus();
                        let _ = window.show();
                    }
                    _ => {
                        println!("Unhandled menu item: {:?}", event.id);
                    }
                });

            // State management
            app.manage(db);
            app.manage(skip_event);

            clipboard::start_clipboard_listener(app.handle().clone())?;
            Ok(())
        })
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_prevent_default::init())
        .invoke_handler(tauri::generate_handler![
            commands::clear_history,
            commands::copy_item,
            commands::load_history,
            commands::paste_item,
            commands::pin_entry,
            commands::remove_entry,
        ])
        .run(tauri::generate_context!())?;

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    if is_dup_instance() {
        return Ok(());
    }

    let args: Vec<String> = std::env::args().collect();
    let silent = if let Some(flag) = args.get(1) {
        flag == "--silent"
    } else {
        false
    };

    #[cfg(windows)]
    std::env::set_var(
        "WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS",
        "--enable-smooth-scrolling",
    );

    run_tauri_app(silent).await?;
    Ok(())
}

fn is_dup_instance() -> bool {
    let sysinfo = sysinfo::System::new_with_specifics(
        RefreshKind::nothing().with_processes(ProcessRefreshKind::everything()),
    );
    for proc in sysinfo.processes_by_name(OsStr::new("squirrel")) {
        if proc.pid().as_u32() != std::process::id() {
            return true;
        }
    }
    false
}
