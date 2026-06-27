//! database and things like that

use crate::clipboard::ClipboardEvent;
use anyhow::{Ok, Result};
use sqlx::{
    sqlite::{SqliteConnectOptions, SqlitePoolOptions},
    SqlitePool,
};
use std::str::FromStr;

#[derive(Debug, Clone)]
pub struct Database {
    pool: SqlitePool,
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
        let entry: ClipboardEvent = sqlx::query_as(
            "
            INSERT INTO clipboard (event_type, content, timestamp, is_pinned) 
            VALUES (?, ?, ?, ?) RETURNING *;
            ",
        )
        .bind(event.event_type.as_str())
        .bind(event.content)
        .bind(event.timestamp)
        .bind(event.is_pinned)
        .fetch_one(&self.pool)
        .await?;
        Ok(entry)
    }

    /// Gets a clipboard entry by its id
    pub async fn get_entry(&self, id: u16) -> Result<ClipboardEvent> {
        let entry: ClipboardEvent = sqlx::query_as("SELECT * FROM clipboard WHERE id = ?;")
            .bind(id)
            .fetch_one(&self.pool)
            .await?;
        Ok(entry)
    }

    /// Removes an entry from the database by its id
    pub async fn remove_entry(&self, id: u16) -> Result<()> {
        sqlx::query("DELETE from clipboard WHERE id=?;")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    /// Gets all clipboard entries from the database, most recent entries first.
    pub async fn get_entries(&self) -> Result<Vec<ClipboardEvent>> {
        let results: Vec<ClipboardEvent> =
            sqlx::query_as("SELECT * FROM clipboard ORDER BY id DESC;")
                .fetch_all(&self.pool)
                .await?;
        Ok(results)
    }

    /// Sets the pinned status of an entry
    pub async fn set_pinned(&self, id: u16, is_pinned: bool) -> Result<()> {
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
