use base64::Engine;
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

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum CbEventContent {
    Text(String),
    /// PNG image bytes
    Image(Vec<u8>),
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
            CbEventContent::Image(bytes) => base64::engine::general_purpose::STANDARD.encode(bytes),
            CbEventContent::File(file) => file.join("\0"),
        }
    }
}

#[derive(Clone, Debug, Serialize, FromRow)]
pub struct ClipboardEvent {
    pub id: i64,
    pub event_type: CbEventType,
    pub content: CbEventContent,
    pub is_pinned: bool,
    pub expires_at: Option<i64>,
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
