//! database and things like that
use crate::{
    clipboard::models::{CbEventContent, CbEventType, ClipboardEvent},
    CONFIG,
};
use anyhow::{Context, Result};
use chrono::{Duration, Local};
use sqlx::{
    sqlite::{SqliteConnectOptions, SqlitePoolOptions},
    SqlitePool,
};
use std::str::FromStr;

#[allow(dead_code)]
const CURRENT_VERSION: i64 = 1;

#[derive(Debug, Clone)]
pub struct Database {
    pool: SqlitePool,
}

impl Database {
    /// Returns a new database instance connected to `{url}/data.db`. This method will clear the
    /// data in the database at the passed URL if one exists _and_ is of an unsupported schema
    /// version.
    pub async fn new(url: &str) -> Result<Self> {
        let options = SqliteConnectOptions::from_str(url)?.create_if_missing(true);
        let pool = SqlitePoolOptions::new().connect_with(options).await?;

        let db_version: (i32,) = sqlx::query_as("PRAGMA user_version;")
            .fetch_one(&pool)
            .await?;

        if db_version.0 == 0 {
            // Pre 1.0 schema, drop it completely. `IF EXISTS` because fresh db will have version 0.
            sqlx::query("DROP TABLE IF EXISTS clipboard; DROP TABLE IF EXISTS _sqlx_migrations;")
                .execute(&pool)
                .await?;
        }

        sqlx::migrate!("./migrations").run(&pool).await?;

        Ok(Self { pool })
    }

    /// Inserts clipboard event content into the database, returning the entry on success or an
    /// error on failure to insert.
    pub async fn create_entry(&self, event: ClipboardEvent) -> Result<ClipboardEvent> {
        let entry: EntryRow = match event.content {
            CbEventContent::Text(text) => {
                sqlx::query_as(
                    "
                    INSERT INTO clipboard (id, event_type, is_pinned, content_text, expires_at)
                    VALUES (?, ?, ?, ?, ?) 
                    RETURNING *;
                    ",
                )
                .bind(event.id)
                .bind(event.event_type)
                .bind(event.is_pinned as u8)
                .bind(&text)
                .bind(event.expires_at)
                .fetch_one(&self.pool)
                .await?
            }

            CbEventContent::Image(bytes) => {
                sqlx::query_as(
                    "
                    INSERT INTO clipboard (id, event_type, is_pinned, content_blob, expires_at)
                    VALUES (?, ?, ?, ?, ?) RETURNING *;
                    ",
                )
                .bind(event.id)
                .bind(event.event_type)
                .bind(event.is_pinned as u8)
                .bind(bytes)
                .bind(event.expires_at)
                .fetch_one(&self.pool)
                .await?
            }

            CbEventContent::File(files) => {
                sqlx::query_as(
                    "
                    INSERT INTO clipboard (id, event_type, is_pinned, content_text, expires_at)
                    VALUES (?, ?, ?, ?, ?) RETURNING *;
                    ",
                )
                .bind(event.id)
                .bind(event.event_type)
                .bind(event.is_pinned as u8)
                .bind(files.join("\0"))
                .bind(event.expires_at)
                .fetch_one(&self.pool)
                .await?
            }
        };

        Ok(entry.into())
    }

    pub async fn get_entry_content(&self, id: i64) -> Result<CbEventContent> {
        let row: EntryContentRow = sqlx::query_as(
            "SELECT event_type, content_text, content_blob FROM clipboard WHERE id = ?;",
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await?;

        let content: CbEventContent = row.try_into()?;

        Ok(content)
    }

    /// Removes an entry from the database by its id
    pub async fn remove_entry(&self, id: i64) -> Result<()> {
        sqlx::query("DELETE from clipboard WHERE id=?;")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    /// Gets all clipboard entries from the database, most recent entries first.
    /// Does NOT get the content for image entries, use [`Database::get_entry_content`].
    pub async fn get_entries(&self) -> Result<Vec<ClipboardEvent>> {
        let results: Vec<EntryRow> = sqlx::query_as(
            "
            SELECT 
                id, 
                event_type, 
                content_text,
                NULL as content_blob,
                is_pinned,
                expires_at
                FROM clipboard 
                ORDER BY id DESC;
            ",
        )
        .fetch_all(&self.pool)
        .await?;
        let results = results.into_iter().map(|x| x.into()).collect();
        Ok(results)
    }

    /// Sets the pinned status of an entry, updating the expires_at to `now + ttl` if unpinning, and
    /// nulling it if pinning.
    pub async fn set_pinned(&self, id: i64, is_pinned: bool) -> Result<()> {
        let new_expires_at = if is_pinned {
            None
        } else {
            Some(
                Local::now().timestamp_micros()
                    + Duration::days(CONFIG.lock().unwrap().history.ttl)
                        .num_microseconds()
                        .context("Config `history_ttl` is too large (UNIX timestamp overflow)")?,
            )
        };

        sqlx::query("UPDATE clipboard SET is_pinned = ?, expires_at = ? WHERE id = ?;")
            .bind(is_pinned as u8)
            .bind(new_expires_at)
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    /// Clears expired entries from the database.
    pub async fn remove_expired(&self) -> Result<()> {
        sqlx::query("DELETE FROM clipboard WHERE expires_at <= ?;")
            .bind(Local::now().timestamp_micros())
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    /// Clears all entries from the database, preserving pins unless `clear_pinned` says otherwise.
    pub async fn clear_entries(&self, clear_pinned: bool) -> Result<()> {
        sqlx::query("DELETE FROM clipboard WHERE is_pinned = FALSE OR ?;")
            .bind(clear_pinned)
            .execute(&self.pool)
            .await?;

        Ok(())
    }
}

#[derive(sqlx::FromRow)]
struct EntryRow {
    pub id: i64,
    pub event_type: CbEventType,
    pub content_text: Option<String>,
    pub content_blob: Option<Vec<u8>>,
    pub is_pinned: bool,
    pub expires_at: Option<i64>,
}

impl From<EntryRow> for ClipboardEvent {
    fn from(row: EntryRow) -> Self {
        let content = match row.event_type {
            CbEventType::Text => CbEventContent::Text(
                row.content_text
                    .expect("Text data not present for text entry"),
            ),

            // unwrap_or_default should mostly be okay for images. the only time content_blob should
            // not be present for an image entry is in `Database::get_entry_content`.
            CbEventType::Image => CbEventContent::Image(row.content_blob.unwrap_or_default()),

            CbEventType::File => CbEventContent::File(
                row.content_text
                    .expect("Text data not present for file entry")
                    .split("\0")
                    .map(|x| x.to_owned())
                    .collect(),
            ),
        };

        Self {
            id: row.id,
            event_type: row.event_type,
            is_pinned: row.is_pinned,
            content,
            expires_at: row.expires_at,
        }
    }
}

#[derive(sqlx::FromRow)]
struct EntryContentRow {
    pub event_type: CbEventType,
    pub content_text: Option<String>,
    pub content_blob: Option<Vec<u8>>,
}

impl TryFrom<EntryContentRow> for CbEventContent {
    type Error = anyhow::Error;
    fn try_from(row: EntryContentRow) -> anyhow::Result<Self> {
        match row.event_type {
            CbEventType::Text => Ok(Self::Text(row.content_text.unwrap())),
            CbEventType::Image => Ok(Self::Image(row.content_blob.unwrap())),
            CbEventType::File => Ok(Self::File(
                row.content_text
                    .unwrap()
                    .split("\0")
                    .map(|x| x.to_owned())
                    .collect(),
            )),
        }
    }
}
