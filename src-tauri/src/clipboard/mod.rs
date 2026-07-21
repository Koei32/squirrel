//! clipboard stuff

pub mod models;
use crate::db::Database;
use anyhow::Result;
use base64::Engine;
use chrono::Local;
use clipboard_rs::{
    common::RustImage, Clipboard, ClipboardHandler, ClipboardWatcher, ClipboardWatcherContext,
    ContentFormat,
};
use models::*;
use std::sync::{
    atomic::{AtomicBool, Ordering::SeqCst},
    Mutex,
};
use tauri::{ipc::Channel, AppHandle, Emitter, Manager};
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
                let hash = self.calculate_hash(&text);
                if hash == self.last_hash.unwrap_or_default() {
                    return;
                }
                self.last_hash = Some(hash);

                let _ = self.tx.send(CbEventContent::Text(text));
            }
        } else if self.ctx.has(ContentFormat::Image) {
            if let Ok(image) = self.ctx.get_image() {
                let img = image.to_png().expect("Failed to convert image data to png");
                let img_bytes = img.get_bytes();

                // Arbitrary 5mb limit
                if img_bytes.len() > 5_000_000 {
                    return;
                }

                // Reject consecutive duplicate copies
                let hash = self.calculate_hash(&img_bytes);
                if hash == self.last_hash.unwrap_or_default() {
                    return;
                }
                self.last_hash = Some(hash);

                let img_b64 = base64::engine::general_purpose::STANDARD.encode(img_bytes);

                let _ = self.tx.send(CbEventContent::Image(img_b64));
            }
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
        // Notify the about the event frontend first
        let _ = app.emit("cb-copy", ClipboardEventNotice::from(&event));
        let event = db.create_entry(event).await?;

        // Then send the content after writing it to db
        let channel = app.state::<Mutex<Option<Channel<ClipboardEvent>>>>();
        let mut lock = channel.lock().unwrap();
        lock.as_mut().unwrap().send(event)?;
    }
    Ok(())
}
