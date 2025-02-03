//! Contains database models and interaction code.
pub mod models;
use crate::constants::db as constants;

/// An alias for the underlying DBMS specific pool type.
pub type ConnectionPool = sqlx::PgPool;

/// Initiate a pooled connection to the database.
pub async fn connect() -> Result<ConnectionPool, errors::DatabaseError> {
    Ok(sqlx::PgPool::connect(&constants::DB_URL).await?)
}

/// Errors returned by functions in this module.
pub mod errors {
    use thiserror::Error;

    /// An error returned by underlying database layer.
    #[derive(Error, Debug)]
    #[error(transparent)]
    pub struct DatabaseError(#[from] sqlx::Error);
}
