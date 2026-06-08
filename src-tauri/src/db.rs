//! database and things like that

use anyhow::Result;
use sqlx::{
    sqlite::{SqliteConnectOptions, SqlitePoolOptions},
    SqlitePool,
};
use std::str::FromStr;
const DATABASE_URL: &str = "sqlite://data.db?mode=rwc";

use crate::clipboard::ClipboardEvent;

#[derive(Debug, Clone)]
pub struct Database {
    pool: SqlitePool,
}

impl Database {
    /// Returns a new database instance connected to `./userdata/data.db`
    pub async fn new() -> Result<Self> {
        let options = SqliteConnectOptions::from_str(DATABASE_URL)?.create_if_missing(true);
        let pool = SqlitePoolOptions::new().connect_with(options).await?;
        sqlx::migrate!("./migrations").run(&pool).await?;

        Ok(Self { pool })
    }

    /// Inserts clipboard event content into the database, returning the event
    /// on success or an error on failure to insert
    pub async fn create_entry(&self, event: ClipboardEvent) -> Result<ClipboardEvent> {
        let entry: ClipboardEvent = sqlx::query_as(
            "
            INSERT INTO clipboard (event_type, content, timestamp) 
            VALUES (?, ?, ?) RETURNING *;
            ",
        )
        .bind(String::from(event.event_type))
        .bind(event.content)
        .bind(event.timestamp)
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

    pub async fn remove_entry(&self, id:u16) -> Result<()> {
        sqlx::query("DELETE from clipboard WHERE id=?;").bind(id).execute(&self.pool).await?;
        Ok(())
    }

    /// Gets all clipboard entries
    pub async fn get_entries(&self) -> Result<Vec<ClipboardEvent>> {
        let results: Vec<ClipboardEvent> = sqlx::query_as("SELECT * FROM clipboard;")
            .fetch_all(&self.pool)
            .await?;
        Ok(results)
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
