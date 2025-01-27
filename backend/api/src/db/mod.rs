//! Contains database models and interaction code.
pub mod models;
use crate::constants::db as constants;

/// Initiate a pooled connection to the database.
pub async fn connect() -> Result<sqlx::PgPool, sqlx::Error> {
    sqlx::PgPool::connect(&constants::DB_URL).await
}
