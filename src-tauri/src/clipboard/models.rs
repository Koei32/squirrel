use clipboard_rs::ClipboardContext;
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use std::hash::{DefaultHasher, Hash, Hasher};
use tauri::AppHandle;
use tokio::sync::mpsc::{self};

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
    pub fn to_type(&self) -> CbEventType {
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
            CbEventContent::Image(b64) => b64,
            CbEventContent::File => todo!(),
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
        ClipboardEventNotice {
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
    pub last_hash: Option<u64>,
    /// App handle
    pub app: AppHandle,
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
    pub fn calculate_hash<T: Hash>(&self, value: &T) -> u64 {
        let mut s = DefaultHasher::new();
        value.hash(&mut s);
        s.finish()
    }
}
