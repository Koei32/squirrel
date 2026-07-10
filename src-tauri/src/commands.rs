use crate::clipboard::models::{CbEventContent, ClipboardEvent};
use crate::db::Database;
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
    id: u32,
) -> std::result::Result<(), String> {
    let content = db.get_entry_content(id);
    let cb = ClipboardContext::new().map_err(|e| e.to_string())?;

    // Skip the next copy event because it's caused by us
    skip.store(true, SeqCst);

    match content.await.map_err(|e| e.to_string())? {
        CbEventContent::Text(text) => cb.set_text(text).map_err(|e| e.to_string())?,
        CbEventContent::Image(_) => todo!("image support"),
        CbEventContent::File => todo!("file support"),
    };
    Ok(())
}

/// Pastes the item associated with the given id via Ctrl+V shortcut emulation
/// using Enigo.
#[tauri::command(async)]
pub async fn paste_item(
    db: State<'_, Database>,
    skip: State<'_, AtomicBool>,
    id: u32,
) -> std::result::Result<(), String> {
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
pub async fn pin_entry(
    db: State<'_, Database>,
    id: u32,
    is_pinned: bool,
) -> std::result::Result<(), String> {
    db.set_pinned(id, is_pinned)
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}

/// Removes the entry associated with the given id.
#[tauri::command(async)]
pub async fn remove_entry(db: State<'_, Database>, id: u32) -> std::result::Result<(), String> {
    db.remove_entry(id).await.map_err(|e| e.to_string())?;
    Ok(())
}

/// Sends the stored clipboard history over the provided channel, most recent
/// entries first.
#[tauri::command]
pub async fn load_history(
    db: State<'_, Database>,
    on_event: Channel<ClipboardEvent>,
) -> std::result::Result<(), String> {
    let events = db.get_entries().await.map_err(|e| e.to_string())?;
    for event in events {
        let _ = on_event.send(event.clone());
    }
    Ok(())
}

/// Clears the entire clipboard history, permanently.
#[tauri::command(async)]
pub async fn clear_history(db: State<'_, Database>) -> std::result::Result<(), String> {
    db.clear_entries().await.map_err(|e| e.to_string())?;
    Ok(())
}
