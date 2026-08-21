use crate::clipboard::models::{CbEventContent, ClipboardEvent};
use crate::db::Database;
use crate::CONFIG;
use clipboard_rs::common::RustImage;
use clipboard_rs::{Clipboard, ClipboardContext};
use enigo::{Direction, Enigo, Key, Keyboard, Settings};
use std::sync::atomic::{AtomicBool, Ordering::SeqCst};
use std::time::Duration;
use tauri::{ipc::Channel, State};
// TODO: make a custom error type to return from tauri commands so that i dont
// have to map_err everywhere like an amateur

/// Copies the item associated with the given id to the clipboard.
#[tauri::command(async)]
pub async fn copy_item(
    db: State<'_, Database>,
    skip: State<'_, AtomicBool>,
    id: i64,
) -> Result<(), String> {
    let content = db.get_entry_content(id);
    let cb = ClipboardContext::new().map_err(|e| e.to_string())?;

    // Skip the next copy event because it's caused by us
    skip.store(true, SeqCst);

    match content.await.map_err(|e| e.to_string())? {
        CbEventContent::Text(text) => cb.set_text(text).map_err(|e| e.to_string())?,
        CbEventContent::Image(bytes) => cb
            .set_image(clipboard_rs::RustImageData::from_bytes(&bytes).map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())?,
        CbEventContent::File(paths) => cb.set_files(paths).map_err(|e| e.to_string())?,
    };
    Ok(())
}

/// Pastes the item associated with the given id via Ctrl+V shortcut emulation
/// using Enigo.
#[tauri::command(async)]
pub async fn paste_item(
    db: State<'_, Database>,
    skip: State<'_, AtomicBool>,
    id: i64,
) -> Result<(), String> {
    copy_item(db, skip, id).await?;
    let mut keyboard = Enigo::new(&Settings::default()).map_err(|e| e.to_string())?;

    // we'll think about mac some other day
    // #[cfg(target_os = "macos")]
    // let modifier = Key::Meta;

    #[cfg(not(target_os = "macos"))]
    let modifier = Key::Control;
    keyboard
        .key(modifier, Direction::Press)
        .map_err(|e| e.to_string())?;
    tokio::time::sleep(Duration::from_millis(5)).await;
    keyboard
        .key(Key::Unicode('v'), Direction::Press)
        .map_err(|e| e.to_string())?;
    tokio::time::sleep(Duration::from_millis(5)).await;
    keyboard
        .key(modifier, Direction::Release)
        .map_err(|e| e.to_string())?;
    keyboard
        .key(Key::Unicode('v'), Direction::Release)
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command(async)]
pub async fn pin_entry(db: State<'_, Database>, id: i64, is_pinned: bool) -> Result<(), String> {
    db.set_pinned(id, is_pinned)
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}

/// Removes the entry associated with the given id.
#[tauri::command(async)]
pub async fn remove_entry(db: State<'_, Database>, id: i64) -> Result<(), String> {
    db.remove_entry(id).await.map_err(|e| e.to_string())?;
    Ok(())
}

/// Returns the content of the entry associated with the given id
#[tauri::command(async)]
pub async fn get_entry_content(db: State<'_, Database>, id: i64) -> Result<String, String> {
    let content = db.get_entry_content(id).await.map_err(|e| e.to_string())?;
    Ok(content.into())
}

/// Sends the stored clipboard history over the provided channel, most recent
/// entries first.
#[tauri::command]
pub async fn load_history(
    db: State<'_, Database>,
    on_event: Channel<ClipboardEvent>,
) -> Result<(), String> {
    let events = db.get_entries().await.map_err(|e| e.to_string())?;
    for event in events {
        let _ = on_event.send(event);
    }
    Ok(())
}

/// Clears the entire clipboard history, permanently.
#[tauri::command(async)]
pub async fn clear_history(db: State<'_, Database>) -> Result<(), String> {
    db.clear_entries().await.map_err(|e| e.to_string())?;
    Ok(())
}

/// Opens the system file explorer with the passed `file` focused.
#[tauri::command]
pub async fn reveal_in_explorer(file: String) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    std::process::Command::new("explorer")
        .arg(format!("/select,{}", file))
        .spawn()
        .map_err(|e| e.to_string())?;

    #[cfg(target_os = "linux")]
    {
        let parent = std::path::Path::new(&file)
            .parent()
            .unwrap_or_else(|| std::path::Path::new("/"));
        std::process::Command::new("xdg-open")
            .arg(parent)
            .spawn()
            .map_err(|e| e.to_string())?;
    }

    Ok(())
}

/// Returns the app theme set by config.
#[tauri::command]
pub async fn get_theme() -> Result<String, String> {
    let theme = CONFIG.lock().unwrap().theme.clone();
    Ok(theme)
}
