//! Contains database models and interaction code.
pub mod models;
use crate::constants::db as constants;

/// An alias for the underlying DBMS specific pool type.
pub type ConnectionPool = sqlx::PgPool;

/// Initiate a pooled connection to the database.
pub async fn connect() -> Result<ConnectionPool, errors::DatabaseError> {
    Ok(sqlx::PgPool::connect(&constants::DB_URL).await?)
}

pub mod errors {
    use std::fmt;

    #[derive(Debug)]
    pub struct DatabaseError(sqlx::Error);

    impl From<sqlx::Error> for DatabaseError {
        fn from(err: sqlx::Error) -> Self {
            Self(err)
        }
    }

    impl std::error::Error for DatabaseError {
        fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
            Some(&self.0)
        }
    }

    impl fmt::Display for DatabaseError {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "Failed to access database ({})", self.0)
        }
    }
}
