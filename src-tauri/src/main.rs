#![warn(clippy::all, clippy::nursery, clippy::cargo)]
#![allow(clippy::multiple_crate_versions)]
// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
mod clipboard;
mod commands;
mod config;
mod db;

use anyhow::Result;
use config::Config;
use db::Database;
use dirs::data_dir;
use std::path::PathBuf;
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, LazyLock, Mutex};
use std::{ffi::OsStr, fs::create_dir_all};
use sysinfo::{ProcessRefreshKind, RefreshKind};
use tauri::{
    image::Image,
    menu::{Menu, MenuItem},
    tray::TrayIconBuilder,
    Builder, Manager,
};
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState};
use tracing::level_filters::LevelFilter;
use tracing::{debug, error, info, warn};
use tracing_subscriber::fmt::format::FmtSpan;

static CONFIG: LazyLock<Arc<Mutex<Config>>> =
    LazyLock::new(|| Arc::new(Mutex::new(Config::default())));

async fn run_tauri_app(silent: bool, mut data_dir: PathBuf) -> Result<()> {
    // Configuration
    data_dir.push("squirrel.toml");
    let config = Config::load(&data_dir)?;
    {
        let mut lock = CONFIG.lock().unwrap();
        *lock = config.clone();
    }
    data_dir.pop();
    info!("Loaded config");

    // Database
    if !data_dir.exists() {
        create_dir_all(&data_dir)?;
    }

    if cfg!(debug_assertions) {
        data_dir.push("data.dev.db");
    } else {
        data_dir.push("data.db");
    }

    let db = Database::new(data_dir.to_str().unwrap()).await?;
    info!("Connected to DB");

    // Whether or not to ignore clipboard events
    let skip_event = AtomicBool::new(false);

    Builder::default()
        .setup(move |app| {
            let window = app
                .get_webview_window("main")
                .expect("Failed to get main webview window");
            // we need this to pass into the tray icon builder
            let window_clone = window.clone();

            if !silent {
                window.show()?;
            } else {
                info!("`--silent` is passed, launching silently");
            }

            // Tray icon setup
            let icon_bytes = include_bytes!("../icons/64x64.png");
            let icon = Image::from_bytes(icon_bytes)?;
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
            debug!("Tray icon initialized");

            // State management
            app.manage(db);
            app.manage(skip_event);

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
            debug!("Launch hotkey registered");

            // Start listener
            clipboard::start_clipboard_listener(app.handle().clone())?;
            info!("Clipboard listener running");
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
            commands::reveal_in_explorer,
            commands::get_theme
        ])
        .run(tauri::generate_context!())?;

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let mut data_dir = data_dir().expect("Failed to get OS data directory");
    data_dir.push("Squirrel");
    if !data_dir.exists() {
        create_dir_all(&data_dir)?;
    }

    data_dir.push("logs");

    let (non_blocking, _guard) = if cfg!(debug_assertions) {
        tracing_appender::non_blocking(std::io::stdout())
    } else {
        tracing_appender::non_blocking(tracing_appender::rolling::daily(&data_dir, "squirrel.log"))
    };
    data_dir.pop();

    let subscriber = tracing_subscriber::fmt();
    subscriber
        .with_span_events(FmtSpan::CLOSE)
        .with_max_level(LevelFilter::DEBUG)
        .with_writer(non_blocking)
        .with_ansi(cfg!(debug_assertions))
        .init();
    debug!("Logging started");

    if is_dup_instance() {
        error!("Another instance of Squirrel is already running, quitting.");
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

    run_tauri_app(silent, data_dir).await?;
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
