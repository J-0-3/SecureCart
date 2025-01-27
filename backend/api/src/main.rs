//! This crate implements the backend API for the `SecureCart` ecommerce platform.

mod constants;
mod controllers;
mod db;
mod routes;
mod utils;

use axum::routing::get;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() {
    let app = axum::Router::new().route("/", get(root));
    let listener = TcpListener::bind("0.0.0.0:8080")
        .await
        .expect("Failed to bind listener");
    axum::serve(listener, app)
        .await
        .expect("Failed to init Axum service");
}

/// The / route is simply used as an availability check.
async fn root() -> String {
    "API is running!".to_owned()
}
