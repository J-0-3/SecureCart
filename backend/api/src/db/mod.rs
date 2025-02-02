//! Contains database models and interaction code.
pub mod models;
use crate::constants::db as constants;

/// An alias for the underlying DBMS specific pool type.
pub type ConnectionPool = sqlx::PgPool;
/// An alias for the underlying DB library specific error type.
pub type StorageError = sqlx::Error;

/// Initiate a pooled connection to the database.
pub async fn connect() -> Result<ConnectionPool, StorageError> {
    sqlx::PgPool::connect(&constants::DB_URL).await
}

