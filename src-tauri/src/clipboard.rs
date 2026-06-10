//! clipboard stuff

use crate::db::Database;
use anyhow::Result;
use chrono::Local;
use clipboard_rs::{
    Clipboard, ClipboardContext, ClipboardHandler, ClipboardWatcher, ClipboardWatcherContext,
};
use enigo::{Direction, Enigo, Key, Keyboard, Settings};
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use std::{
    hash::{DefaultHasher, Hash, Hasher},
    sync::atomic::{AtomicBool, Ordering::SeqCst},
    time::Duration,
};
use tauri::{AppHandle, Emitter, Manager, State};
use tokio::sync::mpsc::{self, UnboundedReceiver};

#[derive(Clone, Debug, Serialize, Deserialize, sqlx::Type, Copy)]
#[sqlx(type_name = "TEXT", rename_all = "snake_case")]
#[serde(rename_all = "camelCase")]
pub enum CbEventType {
    Text,
    Image, // TODO
    File,  // TODO
}

impl From<CbEventType> for String {
    fn from(value: CbEventType) -> Self {
        match value {
            CbEventType::Text => "text".to_string(),
            CbEventType::Image => "image".to_string(),
            CbEventType::File => "file".to_string(),
        }
    }
}

#[derive(Clone, Debug, Serialize, FromRow)]
pub struct ClipboardEvent {
    pub id: u16,
    pub event_type: CbEventType,
    pub content: String,
    pub timestamp: String,
}

/// The minimal payload that will be sent to the frontend. Doesn't have content.
#[derive(Clone, Serialize)]
pub struct ClipboardEventNotice {
    pub id: u16,
    pub event_type: CbEventType,
    pub timestamp: String,
}

impl From<ClipboardEvent> for ClipboardEventNotice {
    fn from(value: ClipboardEvent) -> Self {
        ClipboardEventNotice {
            id: value.id,
            event_type: value.event_type,
            timestamp: value.timestamp,
        }
    }
}

pub struct ClipboardListener {
    ctx: ClipboardContext,
    /// Sender of [`ClipboardEvent`]s
    tx: mpsc::UnboundedSender<ClipboardEvent>,
    /// Content hash of the last event (used to avoid consecutive duplicates)
    last_hash: Option<u64>,
    /// App handle
    app: AppHandle,
}

impl ClipboardListener {
    pub fn new(tx: mpsc::UnboundedSender<ClipboardEvent>, app: AppHandle) -> Self {
        let ctx = ClipboardContext::new().expect("Failed to create clipboard context.");
        ClipboardListener {
            ctx,
            tx,
            last_hash: None,
            app,
        }
    }

    /// Returns the hash of the given value
    fn calculate_hash<T: Hash>(&self, value: &T) -> u64 {
        let mut s = DefaultHasher::new();
        value.hash(&mut s);
        s.finish()
    }
}

impl ClipboardHandler for ClipboardListener {
    fn on_clipboard_change(&mut self) {
        // Flip the skip switch to false and skip if we're supposed to
        if self.app.state::<AtomicBool>().load(SeqCst) {
            self.app.state::<AtomicBool>().store(false, SeqCst);
            return;
        }

        if let Ok(content) = self.ctx.get_text() {
            // Reject whitespace only copy
            if content.trim() == "" {
                return;
            }

            // Reject consecutive duplicate copies
            if self.calculate_hash(&content) == self.last_hash.unwrap_or_default() {
                return;
            }

            let event = ClipboardEvent {
                id: u16::default(),
                event_type: CbEventType::Text,
                content,
                timestamp: Local::now().to_rfc3339(),
            };
            self.last_hash = Some(self.calculate_hash(&event.content));
            let _ = self.tx.send(event);
        }
    }
}

pub fn start_clipboard_listener(app: AppHandle) -> Result<()> {
    let (tx, rx) = mpsc::unbounded_channel::<ClipboardEvent>();
    let app_handle = app.clone();
    // Spawn listnener
    std::thread::spawn(move || {
        let listener = ClipboardListener::new(tx, app);
        let mut watcher =
            ClipboardWatcherContext::new().expect("Failed to create cb watcher context");
        watcher.add_handler(listener).start_watch();
    });

    // Spawn the event emitter
    tokio::spawn(start_emitter(app_handle, rx));
    Ok(())
}

async fn start_emitter(app: AppHandle, mut rx: UnboundedReceiver<ClipboardEvent>) -> Result<()> {
    let db = app.state::<Database>();
    while let Some(event) = rx.recv().await {
        let event = db.create_entry(event).await?;
        let _ = app.emit("cb-text-copy", ClipboardEventNotice::from(event.clone()));
    }
    Ok(())
}

// TODO: make a custom error type to return from tauri commands so that i dont
// have to map_err everywhere like an amateur

/// Copies the item associated with the given id to the clipboard.
#[tauri::command(async)]
pub async fn copy_item(
    db: State<'_, Database>,
    skip: State<'_, AtomicBool>,
    id: u16,
) -> std::result::Result<(), String> {
    let content = db.get_entry(id);
    let cb = ClipboardContext::new().map_err(|e| e.to_string())?;

    // Skip the next copy event because it's caused by us
    skip.store(true, SeqCst);
    cb.set_text(content.await.map_err(|e| e.to_string())?.content)
        .map_err(|e| e.to_string())?;

    Ok(())
}

/// Pastes the item associated with the given id via Ctrl+V shortcut emulation
/// using Enigo.
#[tauri::command(async)]
pub async fn paste_item(
    db: State<'_, Database>,
    skip: State<'_, AtomicBool>,
    id: u16,
) -> std::result::Result<(), String> {
    copy_item(db, skip, id).await?;
    let mut keyboard = Enigo::new(&Settings::default()).map_err(|e| e.to_string())?;

    //  we'll think about mac some other day
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

/// Removes the entry associated with the given id.
#[tauri::command(async)]
pub async fn remove_entry(db: State<'_, Database>, id: u16) -> std::result::Result<(), String> {
    db.remove_entry(id).await.map_err(|e| e.to_string())?;
    Ok(())
}

/// Clears the entire clipboard history, permanently.
#[tauri::command(async)]
pub async fn clear_history(db: State<'_, Database>) -> std::result::Result<(), String> {
    db.clear_entries().await.map_err(|e| e.to_string())?;
    Ok(())
}
