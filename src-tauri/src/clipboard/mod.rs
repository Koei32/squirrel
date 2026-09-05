//! clipboard stuff

pub mod models;
use crate::db::Database;
use crate::CONFIG;
use anyhow::{Context, Result};
use chrono::{Duration, Local};
use clipboard_rs::{
    common::RustImage, Clipboard, ClipboardContext, ClipboardHandler, ClipboardWatcher,
    ClipboardWatcherContext, ContentFormat,
};
use models::*;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::sync::atomic::{AtomicBool, Ordering::SeqCst};
use tauri::Emitter;
use tauri::{AppHandle, Manager};
use tokio::sync::mpsc::{self, UnboundedReceiver};
use tracing::{debug, trace};

impl ClipboardHandler for ClipboardListener {
    fn on_clipboard_change(&mut self) {
        // Skip if we're focused on an ignored program
        if focused_on_ignored() {
            trace!("Event ignored because focused on ignored program");
            return;
        }

        // Skip if we're supposed to while flipping the switch
        if self.app.state::<AtomicBool>().load(SeqCst) {
            self.app.state::<AtomicBool>().store(false, SeqCst);
            trace!("Event skipped");
            return;
        }

        if self.ctx.has(ContentFormat::Text) && CONFIG.lock().unwrap().capture.text {
            if let Ok(text) = self.ctx.get_text() {
                // Reject whitespace only copy
                if text.trim().is_empty() {
                    return;
                }
                debug!(len = text.len(), "Text copy event detected");

                // Reject consecutive duplicate copies
                let hash = calculate_hash(&text);
                let mut last = self.last_hash.lock().unwrap();
                if Some(hash) == *last {
                    debug!("Skipped duplicate event");
                    return;
                }
                *last = Some(hash);
                drop(last);

                let _ = self.tx.send(CbEventContent::Text(text));
            }
        } else if self.ctx.has(ContentFormat::Image) && CONFIG.lock().unwrap().capture.images {
            let tx = self.tx.clone();
            let last_hash = self.last_hash.clone();
            let max_image_size = CONFIG.lock().unwrap().capture.max_image_size as usize;

            tauri::async_runtime::spawn(async move {
                let result = tokio::task::spawn_blocking(move || -> Option<Vec<u8>> {
                    let ctx = ClipboardContext::new().ok()?;
                    let img = ctx.get_image().ok()?;
                    let png = img.to_png().ok()?;
                    let img_bytes = png.get_bytes().to_vec();

                    debug!(size = img_bytes.len(), "Image copy event detected");

                    if img_bytes.len() > max_image_size {
                        debug!(
                            max_image_size,
                            img_size = img_bytes.len(),
                            "Skipped image because it is larger than max_image_size"
                        );
                        return None;
                    }

                    let hash = calculate_hash(&img_bytes);

                    let mut last = last_hash.lock().unwrap();
                    if Some(hash) == *last {
                        debug!("Skipped duplicate event");
                        return None;
                    }
                    *last = Some(hash);
                    drop(last);

                    Some(img_bytes)
                })
                .await;

                if let Ok(Some(bytes)) = result {
                    let _ = tx.send(CbEventContent::Image(bytes));
                }
            });
        } else if self.ctx.has(ContentFormat::Files) && CONFIG.lock().unwrap().capture.files {
            if let Ok(paths) = self.ctx.get_files() {
                debug!(num_files = paths.len(), "File(s) copy event detected");

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
    debug!("Spawned clipboard listener thread");

    // Spawn the event emitter
    debug!("Spawning event emitter task...");
    tokio::spawn(start_emitter(app_handle, rx));
    Ok(())
}

async fn start_emitter(app: AppHandle, mut rx: UnboundedReceiver<CbEventContent>) -> Result<()> {
    let db = app.state::<Database>();

    while let Some(content) = rx.recv().await {
        let timestamp = Local::now().timestamp_micros();
        let event = ClipboardEvent {
            id: timestamp,
            event_type: content.to_type(),
            content,
            is_pinned: false,
            expires_at: Some(
                timestamp
                + Duration::days(CONFIG.lock().unwrap().history.ttl)
                .num_microseconds()
                .context("Config `history_ttl` is too large (UNIX timestamp overflow)")?,
            ),
        };
        debug!(id = timestamp, "Constructed ClipboardEvent");
        
        // Send to frontend
        app.emit("cb-copy", event.clone())?;
        debug!(id = timestamp, "Sent to frontend");
        
        db.create_entry(event).await?;
        debug!(id = timestamp, "Wrote to db");
    }

    Ok(())
}

/// Checks whether the currently focused program is ignored by config
pub fn focused_on_ignored() -> bool {
    let window =
        active_win_pos_rs::get_active_window().expect("There should always be a window focused");
    let program_name = window
        .process_path
        .file_name()
        .expect("ActiveWindow always has a path that leads to an executable")
        .to_str()
        .unwrap();

    CONFIG
        .lock()
        .unwrap()
        .app
        .ignore
        .iter()
        .any(|name| name == program_name)
}

/// Returns the hash of the given value
pub fn calculate_hash<T: Hash>(value: &T) -> u64 {
    let mut s = DefaultHasher::new();
    value.hash(&mut s);
    s.finish()
}
