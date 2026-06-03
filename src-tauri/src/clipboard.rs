//! clipboard stuff

use anyhow::Result;
use clipboard_rs::{
    Clipboard, ClipboardContext, ClipboardHandler, ClipboardWatcher, ClipboardWatcherContext,
};
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use tauri::{AppHandle, Emitter, Manager};
use tokio::sync::mpsc::{self, UnboundedReceiver};

use crate::AppState;

#[derive(Clone, Serialize, Deserialize, sqlx::Type, Copy)]
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

#[derive(Clone, Serialize, FromRow)]
pub struct ClipboardEvent {
    pub event_type: CbEventType,
    pub content: String,
}

pub struct ClipboardListener {
    ctx: ClipboardContext,
    tx: mpsc::UnboundedSender<ClipboardEvent>,
}

impl ClipboardListener {
    pub fn new(tx: mpsc::UnboundedSender<ClipboardEvent>) -> Self {
        let ctx = ClipboardContext::new().expect("Failed to create clipboard context.");
        ClipboardListener { ctx, tx }
    }
}

impl ClipboardHandler for ClipboardListener {
    fn on_clipboard_change(&mut self) {
        if let Ok(_) = self.ctx.get_text() {
            let _ = self.tx.send(ClipboardEvent {
                event_type: CbEventType::Text,
                content: "".to_string(),
            });
        }
    }
}

pub fn start_clipboard_listener(app: AppHandle) -> Result<()> {
    let (tx, rx) = mpsc::unbounded_channel::<ClipboardEvent>();

    // spawn listnener
    std::thread::spawn(move || {
        let listener = ClipboardListener::new(tx);
        let mut watcher =
            ClipboardWatcherContext::new().expect("Failed to create cb watcher context");
        watcher.add_handler(listener).start_watch();
    });

    // spawn the event emitter
    tokio::spawn(start_emitter(app, rx));
    Ok(())
}

async fn start_emitter(app: AppHandle, mut rx: UnboundedReceiver<ClipboardEvent>) -> Result<()> {
    let state = app.state::<AppState>();
    while let Some(event) = rx.recv().await {
        let _ = app.emit("cb-text-copy", event.event_type);
        println!("emitted event");
        state.db.create_entry(event).await?;
        println!("added to db");
    }
    Ok(())
}

#[tauri::command]
pub fn copy_content(app: tauri::AppHandle, window: tauri::Window, content: String) {
    println!("copy_content called");
}
