//! database and things like that
use crate::clipboard::models::{CbEventContent, CbEventType, ClipboardEvent};
use anyhow::Result;
use sqlx::{
    sqlite::{SqliteConnectOptions, SqlitePoolOptions},
    SqlitePool,
};
use std::str::FromStr;

#[derive(Debug, Clone)]
pub struct Database {
    pool: SqlitePool,
}

#[derive(sqlx::FromRow)]
struct ClipboardEntryRow {
    pub id: u32,
    pub event_type: CbEventType,
    pub content: Option<String>,
    pub timestamp: String,
    pub is_pinned: bool,
}

#[allow(clippy::fallible_impl_from)]
impl From<ClipboardEntryRow> for ClipboardEvent {
    fn from(row: ClipboardEntryRow) -> Self {
        let content = match row.event_type {
            CbEventType::Text => CbEventContent::Text(row.content.unwrap()),
            CbEventType::Image => CbEventContent::Image(row.content.unwrap_or_default()),
            CbEventType::File => CbEventContent::File(
                row.content
                    .unwrap()
                    .split("\0")
                    .map(|x| x.to_owned())
                    .collect(),
            ),
        };

        Self {
            id: row.id,
            event_type: row.event_type,
            is_pinned: row.is_pinned,
            timestamp: row.timestamp,
            content,
        }
    }
}

#[derive(sqlx::FromRow)]
struct CbEntryContentRow {
    pub event_type: CbEventType,
    pub content: String,
}

impl From<CbEntryContentRow> for CbEventContent {
    fn from(row: CbEntryContentRow) -> Self {
        match row.event_type {
            CbEventType::Text => Self::Text(row.content),
            CbEventType::Image => Self::Image(row.content),
            CbEventType::File => {
                Self::File(row.content.split("\0").map(|x| x.to_owned()).collect())
            }
        }
    }
}

impl Database {
    /// Returns a new database instance connected to `{url}/data.db`
    pub async fn new(url: &str) -> Result<Self> {
        let options = SqliteConnectOptions::from_str(url)?.create_if_missing(true);
        let pool = SqlitePoolOptions::new().connect_with(options).await?;
        sqlx::migrate!("./migrations").run(&pool).await?;

        Ok(Self { pool })
    }

    /// Inserts clipboard event content into the database, returning the event
    /// on success or an error on failure to insert
    pub async fn create_entry(&self, event: ClipboardEvent) -> Result<ClipboardEvent> {
        let entry: ClipboardEntryRow = sqlx::query_as(
            "
            INSERT INTO clipboard (event_type, content, timestamp, is_pinned) 
            VALUES (?, ?, ?, ?) RETURNING *;
            ",
        )
        .bind(event.event_type.as_str())
        .bind(String::from(event.content).as_str())
        .bind(event.timestamp)
        .bind(event.is_pinned)
        .fetch_one(&self.pool)
        .await?;

        Ok(entry.into())
    }

    /// Gets a clipboard entry by its id
    #[allow(dead_code)]
    pub async fn get_entry(&self, id: u32) -> Result<ClipboardEvent> {
        let entry: ClipboardEntryRow = sqlx::query_as("SELECT * FROM clipboard WHERE id = ?;")
            .bind(id)
            .fetch_one(&self.pool)
            .await?;
        Ok(entry.into())
    }

    pub async fn get_entry_content(&self, id: u32) -> Result<CbEventContent> {
        let row: CbEntryContentRow =
            sqlx::query_as("SELECT event_type, content FROM clipboard WHERE id = ?;")
                .bind(id)
                .fetch_one(&self.pool)
                .await?;

        let content: CbEventContent = row.into();

        Ok(content)
    }

    /// Removes an entry from the database by its id
    pub async fn remove_entry(&self, id: u32) -> Result<()> {
        sqlx::query("DELETE from clipboard WHERE id=?;")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    /// Gets all clipboard entries from the database, most recent entries first.
    /// Does NOT get the content for image entries, use [`Database::get_entry_content`].
    pub async fn get_entries(&self) -> Result<Vec<ClipboardEvent>> {
        let results: Vec<ClipboardEntryRow> = sqlx::query_as(
            "
            SELECT 
                id, 
                event_type, 
                CASE WHEN event_type = 'text' OR event_type = 'file' THEN content ELSE NULL END as content,
                timestamp,
                is_pinned
                FROM clipboard 
                ORDER BY id DESC;
            ",
        )
        .fetch_all(&self.pool)
        .await?;
        let results = results.into_iter().map(|x| x.into()).collect();
        Ok(results)
    }

    /// Sets the pinned status of an entry
    pub async fn set_pinned(&self, id: u32, is_pinned: bool) -> Result<()> {
        sqlx::query("UPDATE clipboard SET is_pinned = ? WHERE id = ?;")
            .bind(is_pinned as u8)
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    /// Clears the whole database. (!)
    pub async fn clear_entries(&self) -> Result<()> {
        sqlx::query(
            "
            DELETE FROM clipboard;
        ",
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}
