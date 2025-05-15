use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

use crate::db::errors::DatabaseError;

pub struct HttpError {
    status: StatusCode,
    message: Option<String>,
}

impl From<StatusCode> for HttpError {
    fn from(err: StatusCode) -> Self {
        Self {
            status: err,
            message: None,
        }
    }
}

impl HttpError {
    pub const fn new(status: StatusCode, message: Option<String>) -> Self {
        Self { status, message }
    }
}

impl IntoResponse for HttpError {
    fn into_response(self) -> Response {
        let message = self
            .message
            .unwrap_or_else(|| self.status.canonical_reason().unwrap_or("").to_owned());
        (self.status, Json(json!({"message": message}))).into_response()
    }
}

impl From<DatabaseError> for HttpError {
    fn from(err: DatabaseError) -> Self {
        eprintln!("Error raised from database in handler: {err}");
        Self::new(StatusCode::INTERNAL_SERVER_ERROR, Some(err.to_string()))
    }
}
