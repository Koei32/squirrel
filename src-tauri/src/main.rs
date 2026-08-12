#![warn(clippy::all, clippy::nursery, clippy::cargo)]
#![allow(clippy::multiple_crate_versions)]
// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
mod clipboard;
mod commands;
mod config;
mod db;

use crate::clipboard::models::ClipboardEvent;
use anyhow::Result;
use config::Config;
use db::Database;
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, LazyLock, Mutex};
use std::{ffi::OsStr, fs::create_dir_all};
use sysinfo::{ProcessRefreshKind, RefreshKind};
use tauri::ipc::Channel;
use tauri::{
    image::Image,
    menu::{Menu, MenuItem},
    tray::TrayIconBuilder,
    Builder, Manager,
};
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState};

static CONFIG: LazyLock<Arc<Mutex<Config>>> =
    LazyLock::new(|| Arc::new(Mutex::new(Config::default())));

async fn run_tauri_app(silent: bool) -> Result<()> {
    let mut data_dir = dirs::data_dir().expect("Failed to get data directory");
    data_dir.push("Squirrel");
    if !data_dir.exists() {
        create_dir_all(&data_dir)?;
    }

    let mut cfg_path = data_dir;
    cfg_path.push("squirrel.toml");
    let config = Config::load(&cfg_path)?;

    {
        let mut lock = CONFIG.lock().unwrap();
        *lock = config.clone();
    }

    let mut db_url = cfg_path;
    db_url.pop();
    db_url.push("data.db");
    let db = Database::new(db_url.to_str().unwrap()).await?;

    // Whether or not to ignore clipboard events
    let skip_event = AtomicBool::new(false);

    // Channel over which events are sent
    let event_channel: Mutex<Option<Channel<ClipboardEvent>>> = Mutex::new(None);

    Builder::default()
        .setup(move |app| {
            let window = app
                .get_webview_window("main")
                .expect("Failed to get main webview window");
            // we need this to pass into the tray icon builder
            let window_clone = window.clone();

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
                        let _ = window_clone.set_focus();
                        let _ = window_clone.show();
                    }
                    _ => {
                        println!("Unhandled menu item: {:?}", event.id);
                    }
                });

            // State management
            app.manage(db);
            app.manage(skip_event);
            app.manage(event_channel);

            // Global launch hotkey
            let launch_hotkey =
                Shortcut::new(Some(Modifiers::CONTROL | Modifiers::SHIFT), Code::KeyV);
            app.handle().plugin(
                tauri_plugin_global_shortcut::Builder::new()
                    .with_handler(move |_app, shortcut, event| {
                        if shortcut == &launch_hotkey && event.state() == ShortcutState::Pressed {
                            let _ = window.show();
                            let _ = window.unminimize();
                            let _ = window.set_focus();
                        }
                    })
                    .build(),
            )?;
            app.global_shortcut().register(launch_hotkey)?;

            // Start listener
            clipboard::start_clipboard_listener(app.handle().clone())?;
            Ok(())
        })
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_prevent_default::init())
        .invoke_handler(tauri::generate_handler![
            commands::clear_history,
            commands::copy_item,
            commands::load_history,
            commands::paste_item,
            commands::pin_entry,
            commands::remove_entry,
            commands::get_entry_content,
            commands::set_event_channel,
            commands::reveal_in_explorer
        ])
        .run(tauri::generate_context!())?;

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    if is_dup_instance() {
        return Ok(());
    }

    // Whether we want to keep the window hidden at launch
    let silent: bool = std::env::args().any(|arg| &arg == "--silent");

    // WebView2 experimental smooth scrolling flag, might remove later
    #[cfg(windows)]
    std::env::set_var(
        "WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS",
        "--enable-smooth-scrolling",
    );

    run_tauri_app(silent).await?;
    Ok(())
}

/// Returns true if a process named `squirrel` is already running on the system.
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
