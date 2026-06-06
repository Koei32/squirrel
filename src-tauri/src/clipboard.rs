//! clipboard stuff

use crate::AppState;
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
    sync::atomic::Ordering::SeqCst,
};
use tauri::{AppHandle, Emitter, Manager};
use tokio::sync::mpsc::{self, UnboundedReceiver};

#[derive(Clone, Debug, Serialize, Deserialize, sqlx::Type, Copy)]
#[sqlx(type_name = "TEXT", rename_all = "snake_case")]
#[serde(rename_all = "camelCase")]
pub enum CbEventType {
    Text,
    Image,
    File,
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
    pub event_type: CbEventType,
    pub content: String,
    pub timestamp: String,
}

/// The minimal payload that will be sent to the frontend.
/// Only contains the event type and not the content.
#[derive(Clone, Serialize)]
pub struct ClipboardEventNotice {
    pub event_type: CbEventType,
    pub timestamp: String,
}

impl From<ClipboardEvent> for ClipboardEventNotice {
    fn from(value: ClipboardEvent) -> Self {
        ClipboardEventNotice {
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
        if self.app.state::<AppState>().skip_event.load(SeqCst) {
            self.app.state::<AppState>().skip_event.store(false, SeqCst);
            return;
        }

        if let Ok(content) = self.ctx.get_text() {
            if content.trim() == "" {
                return;
            }

            // if the copied item was the same as the last item, do nothing
            if self.calculate_hash(&content) == self.last_hash.unwrap_or_default() {
                return;
            }

            let event = ClipboardEvent {
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
    // spawn listnener
    std::thread::spawn(move || {
        let listener = ClipboardListener::new(tx, app);
        let mut watcher =
            ClipboardWatcherContext::new().expect("Failed to create cb watcher context");
        watcher.add_handler(listener).start_watch();
    });

    // spawn the event emitter
    tokio::spawn(start_emitter(app_handle, rx));
    Ok(())
}

async fn start_emitter(app: AppHandle, mut rx: UnboundedReceiver<ClipboardEvent>) -> Result<()> {
    let state = app.state::<AppState>();
    while let Some(event) = rx.recv().await {
        let _ = app.emit("cb-text-copy", ClipboardEventNotice::from(event.clone()));
        state
            .db
            .create_entry(event)
            .await
            .expect("failed creating entry");
    }
    Ok(())
}

// TODO: make a custom error type to return from tauri commands so that i dont
// have to map_err everywhere like an amateur

#[tauri::command]
pub fn copy_item(app: tauri::AppHandle, content: String) -> std::result::Result<(), String> {
    app.state::<AppState>().skip_event.store(true, SeqCst);
    let cb = ClipboardContext::new().map_err(|e| e.to_string())?;
    cb.set_text(content).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn paste_item(app: tauri::AppHandle, content: String) -> std::result::Result<(), String> {
    println!("paste_content called");
    app.state::<AppState>().skip_event.store(true, SeqCst);
    let mut keyboard = Enigo::new(&Settings::default()).map_err(|e| e.to_string())?;
    let cb = ClipboardContext::new().map_err(|e| e.to_string())?;
    cb.set_text(content).map_err(|e| e.to_string())?;

    //  we'll think about mac some other day
    // #[cfg(target_os = "macos")]
    // let modifier = Key::Meta;

    #[cfg(not(target_os = "macos"))]
    let modifier = Key::Control;
    keyboard
        .key(modifier, Direction::Press)
        .map_err(|e| e.to_string())?;
    keyboard
        .key(Key::Unicode('v'), Direction::Press)
        .map_err(|e| e.to_string())?;
    keyboard
        .key(Key::Unicode('v'), Direction::Release)
        .map_err(|e| e.to_string())?;
    keyboard
        .key(modifier, Direction::Release)
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command(async)]
pub async fn clear_history(app: tauri::AppHandle) -> std::result::Result<(), String> {
    app.state::<AppState>()
        .db
        .clear_entries()
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}
