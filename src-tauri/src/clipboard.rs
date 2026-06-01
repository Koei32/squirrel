//! clipboard stuff
use clipboard_rs::{
    Clipboard, ClipboardContext, ClipboardHandler, ClipboardWatcher, ClipboardWatcherContext,
};
use serde::Serialize;
use tauri::{AppHandle, Emitter};
use tokio::sync::mpsc;

// pub trait ClipboardEvent {}

#[derive(Clone, Serialize)]
pub enum CbEventType {
    Text,
    Image,
    File,
}

#[derive(Clone, Serialize)]
pub struct ClipboardEvent {
    event_type: CbEventType,
    content: String,
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
        let content_type = self
            .ctx
            .available_formats()
            .expect("no content on clipboard");
        dbg!(content_type);

        if let Ok(content) = self.ctx.get_text() {
            let _ = self.tx.send(ClipboardEvent {
                event_type: CbEventType::Text,
                content,
            });
        }
    }
}

pub fn start_clipboard_listener(app: AppHandle) {
    let (tx, mut rx) = mpsc::unbounded_channel::<ClipboardEvent>();

    std::thread::spawn(move || {
        let listener = ClipboardListener::new(tx);
        let mut watcher =
            ClipboardWatcherContext::new().expect("Failed to create cb watcher context");
        watcher.add_handler(listener).start_watch();
    });

    tauri::async_runtime::spawn(async move {
        while let Some(event) = rx.recv().await {
            println!("{}", event.content);
            let _ = app.emit("cb-text-copy", event);
        }
    });
}
