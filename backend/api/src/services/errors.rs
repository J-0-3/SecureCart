//! Shared errors used in multiple services.
use crate::{db::errors::DatabaseError, services::sessions::errors::SessionStorageError};
use thiserror::Error;

/// Errors returned by underlying storage layers.
#[derive(Error, Debug)]
pub enum StorageError {
    /// An error returned by the database.
    #[error(transparent)]
    DatabaseError(#[from] DatabaseError),
    /// An error returned by the session store.
    #[error(transparent)]
    SessionStorageError(#[from] SessionStorageError),
}
