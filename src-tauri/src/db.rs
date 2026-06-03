//! database and things like that

use std::str::FromStr;

use anyhow::{Ok, Result};
use sqlx::{
    sqlite::{SqliteConnectOptions, SqlitePoolOptions},
    ConnectOptions, SqlitePool,
};
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
    pub async fn create_entry(&self, event: ClipboardEvent) -> Result<()> {
        sqlx::query(
            "
            INSERT INTO clipboard (type, content) 
            VALUES (?, ?);
            ",
        )
        .bind(String::from(event.event_type))
        .bind(event.content)
        .execute(&self.pool)
        .await
        .expect("failed insertion");

        Ok(())
    }

    /// Gets all clipboard entries
    pub async fn get_entries(&self) -> Result<Vec<ClipboardEvent>> {
        let results: Vec<ClipboardEvent> = sqlx::query_as("SELECT * FROM clipboard;")
            .fetch_all(&self.pool)
            .await?;
        Ok(results)
    }
}
