//! This crate implements the backend API for the `SecureCart` ecommerce platform.

mod constants;
mod controllers;
mod db;
mod routes;
mod utils;

use axum::{routing::get, extract::Json};
use tokio::net::TcpListener;

#[tokio::main]
async fn main() {
    let app = axum::Router::new()
        .route("/", get(root))
        .nest("/auth", routes::auth::create_router());
    let listener = TcpListener::bind("0.0.0.0:8080")
        .await
        .expect("Failed to bind listener");
    axum::serve(listener, app)
        .await
        .expect("Failed to init Axum service");
}

/// The / route is simply used as an availability check.
async fn root() -> Json<String> {
    Json("API is running!".to_owned())
}
