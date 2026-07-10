//! clipboard stuff

pub mod models;
use crate::db::Database;
use anyhow::Result;
use chrono::Local;
use clipboard_rs::{
    Clipboard, ClipboardHandler, ClipboardWatcher, ClipboardWatcherContext, ContentFormat,
};
use models::*;
use std::sync::atomic::{AtomicBool, Ordering::SeqCst};
use tauri::{AppHandle, Emitter, Manager};
use tokio::sync::mpsc::{self, UnboundedReceiver};

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
