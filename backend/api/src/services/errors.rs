//! Shared errors used in multiple services.
use crate::{
    db::errors::DatabaseError, services::sessions::errors::SessionStorageError,
    utils::httperror::HttpError,
};
use axum::http::StatusCode;
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

impl From<StorageError> for HttpError {
    fn from(value: StorageError) -> Self {
        eprintln!("Storage error in route handler: {value}");
        Self::new(StatusCode::INTERNAL_SERVER_ERROR, Some(value.to_string()))
    }
}
