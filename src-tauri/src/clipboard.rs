//! clipboard stuff

use crate::db::Database;
use anyhow::Result;
use chrono::Local;
use clipboard_rs::{
    Clipboard, ClipboardContext, ClipboardHandler, ClipboardWatcher, ClipboardWatcherContext,
    ContentFormat,
};
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use std::{
    hash::{DefaultHasher, Hash, Hasher},
    sync::atomic::{AtomicBool, Ordering::SeqCst},
};
use tauri::{AppHandle, Emitter, Manager};
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

impl CbEventType {
    pub fn as_str(&self) -> &'static str {
        match self {
            CbEventType::Text => "text",
            CbEventType::Image => "image",
            CbEventType::File => "file",
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum CbEventContent {
    Text(String),
    Image(String), // base64 encoded
    File,          // todo
}

impl CbEventContent {
    fn to_type(&self) -> CbEventType {
        match self {
            CbEventContent::Text(_) => CbEventType::Text,
            CbEventContent::Image(_) => CbEventType::Image,
            CbEventContent::File => CbEventType::File,
        }
    }
}

impl From<CbEventContent> for String {
    fn from(value: CbEventContent) -> Self {
        match value {
            CbEventContent::Text(x) => x,
            _ => "".to_string(),
        }
    }
}

#[derive(Clone, Debug, Serialize, FromRow)]
pub struct ClipboardEvent {
    pub id: u32,
    pub event_type: CbEventType,
    pub content: CbEventContent,
    pub timestamp: String,
    pub is_pinned: bool,
}

/// The minimal payload that will be sent to the frontend. Doesn't have content.
#[derive(Clone, Serialize)]
pub struct ClipboardEventNotice {
    pub id: u32,
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
    tx: mpsc::UnboundedSender<CbEventContent>,
    /// Content hash of the last event (used to avoid consecutive duplicates)
    last_hash: Option<u64>,
    /// App handle
    app: AppHandle,
}

impl ClipboardListener {
    pub fn new(tx: mpsc::UnboundedSender<CbEventContent>, app: AppHandle) -> Self {
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
        // Skip if we're supposed to while flipping the switch
        if self.app.state::<AtomicBool>().load(SeqCst) {
            self.app.state::<AtomicBool>().store(false, SeqCst);
            return;
        }

        if self.ctx.has(ContentFormat::Text) {
            if let Ok(text) = self.ctx.get_text() {
                // Reject whitespace only copy
                if text.trim().is_empty() {
                    return;
                }

                // Reject consecutive duplicate copies
                if self.calculate_hash(&text) == self.last_hash.unwrap_or_default() {
                    return;
                }

                self.last_hash = Some(self.calculate_hash(&text));
                let _ = self.tx.send(CbEventContent::Text(text));
            }
        } else if self.ctx.has(ContentFormat::Image) {
            todo!("images are not handled yet");
        }
    }
}

pub fn start_clipboard_listener(app: AppHandle) -> Result<()> {
    let (tx, rx) = mpsc::unbounded_channel::<CbEventContent>();
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

async fn start_emitter(app: AppHandle, mut rx: UnboundedReceiver<CbEventContent>) -> Result<()> {
    let db = app.state::<Database>();
    while let Some(content) = rx.recv().await {
        let event = ClipboardEvent {
            id: u32::default(),
            event_type: content.to_type(),
            content,
            timestamp: Local::now().to_rfc3339(),
            is_pinned: false,
        };
        let event = db.create_entry(event).await?;
        let _ = app.emit("cb-text-copy", ClipboardEventNotice::from(event.clone()));
    }
    Ok(())
}
