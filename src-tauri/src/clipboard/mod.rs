//! clipboard stuff

pub mod models;
use crate::db::Database;
use crate::CONFIG;
use anyhow::Result;
use base64::Engine;
use chrono::Local;
use clipboard_rs::{
    common::RustImage, Clipboard, ClipboardContext, ClipboardHandler, ClipboardWatcher,
    ClipboardWatcherContext, ContentFormat,
};
use models::*;
use std::hash::{DefaultHasher, Hash, Hasher};
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

        // TODO: maybe make the ClipboardListener hold copies of relevant config items so as to not
        // call app.state() every time.

        if self.ctx.has(ContentFormat::Text) && CONFIG.lock().unwrap().capture.text {
            if let Ok(text) = self.ctx.get_text() {
                // Reject whitespace only copy
                if text.trim().is_empty() {
                    return;
                }

                // Reject consecutive duplicate copies
                let hash = calculate_hash(&text);
                let mut last = self.last_hash.lock().unwrap();
                if Some(hash) == *last {
                    return;
                }
                *last = Some(hash);
                drop(last);

                let _ = self.tx.send(CbEventContent::Text(text));
            }
        } else if self.ctx.has(ContentFormat::Image) && CONFIG.lock().unwrap().capture.images {
            let tx = self.tx.clone();
            let last_hash = self.last_hash.clone();
            let max_image_size = CONFIG.lock().unwrap().max_image_size as usize;

            tauri::async_runtime::spawn(async move {
                let result = tokio::task::spawn_blocking(move || -> Option<String> {
                    let ctx = ClipboardContext::new().ok()?;
                    let img = ctx.get_image().ok()?;
                    let png = img.to_png().ok()?;
                    let img_bytes = png.get_bytes();

                    if img_bytes.len() > max_image_size {
                        return None;
                    }

                    let hash = calculate_hash(&img_bytes);

                    let mut last = last_hash.lock().unwrap();
                    if Some(hash) == *last {
                        return None;
                    }
                    *last = Some(hash);
                    drop(last);

                    let b64 = base64::engine::general_purpose::STANDARD.encode(img_bytes);
                    Some(b64)
                })
                .await;

                if let Ok(Some(b64)) = result {
                    let _ = tx.send(CbEventContent::Image(b64));
                }
            });
        } else if self.ctx.has(ContentFormat::Files) && CONFIG.lock().unwrap().capture.files {
            if let Ok(paths) = self.ctx.get_files() {
                let hash = calculate_hash(&paths);
                let mut last = self.last_hash.lock().unwrap();
                if Some(hash) == *last {
                    return;
                }
                *last = Some(hash);
                drop(last);

                let _ = self.tx.send(CbEventContent::File(paths));
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

        // Notify the frontend about the event first
        let _ = app.emit("cb-copy", ClipboardEventNotice::from(&event));

        // Then send the content after writing it to db
        let event = db.create_entry(event).await?;
        let channel = app.state::<Mutex<Option<Channel<ClipboardEvent>>>>();
        let mut lock = channel.lock().unwrap();
        lock.as_mut().unwrap().send(event)?;
    }
    Ok(())
}

/// Returns the hash of the given value
pub fn calculate_hash<T: Hash>(value: &T) -> u64 {
    let mut s = DefaultHasher::new();
    value.hash(&mut s);
    s.finish()
}
