use clipboard_rs::ClipboardContext;
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use std::sync::{Arc, Mutex};
use tauri::AppHandle;
use tokio::sync::mpsc::{self};

#[derive(Clone, Debug, Serialize, Deserialize, sqlx::Type, Copy)]
#[sqlx(type_name = "TEXT", rename_all = "camelCase")]
#[serde(rename_all = "PascalCase")]
pub enum CbEventType {
    Text,
    Image,
    File,
}

impl CbEventType {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Text => "text",
            Self::Image => "image",
            Self::File => "file",
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum CbEventContent {
    Text(String),
    /// Base64 encoded PNG image data
    Image(String),
    /// A list of file paths
    File(Vec<String>),
}

impl CbEventContent {
    pub const fn to_type(&self) -> CbEventType {
        match self {
            Self::Text(_) => CbEventType::Text,
            Self::Image(_) => CbEventType::Image,
            Self::File(_) => CbEventType::File,
        }
    }
}

impl From<CbEventContent> for String {
    fn from(value: CbEventContent) -> Self {
        match value {
            CbEventContent::Text(text) => text,
            CbEventContent::Image(b64) => b64,
            CbEventContent::File(file) => file.join("\0"),
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

impl From<&ClipboardEvent> for ClipboardEventNotice {
    fn from(value: &ClipboardEvent) -> Self {
        Self {
            id: value.id,
            event_type: value.event_type,
            timestamp: value.timestamp.clone(),
        }
    }
}

pub struct ClipboardListener {
    pub ctx: ClipboardContext,
    /// Sender of [`CbEventContent`]s
    pub tx: mpsc::UnboundedSender<CbEventContent>,
    /// Content hash of the last event (used to avoid consecutive duplicates)
    pub last_hash: Arc<Mutex<Option<u64>>>,
    /// App handle
    pub app: AppHandle,
}

impl ClipboardListener {
    pub fn new(tx: mpsc::UnboundedSender<CbEventContent>, app: AppHandle) -> Self {
        let ctx = ClipboardContext::new().expect("Failed to create clipboard context.");
        Self {
            ctx,
            tx,
            last_hash: Arc::new(Mutex::new(None)),
            app,
        }
    }
}
