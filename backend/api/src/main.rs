//! This crate implements the backend API for the `SecureCart` ecommerce platform.

mod constants;
mod db;
mod middleware;
mod routes;
mod services;
mod state;
mod utils;

use axum::{extract::Json, routing::get};
use tokio::net::TcpListener;

#[tokio::main]
async fn main() {
    let db_conn = db::connect()
        .await
        .expect("Could not connect to primary databasee");
    let session_store_conn = services::sessions::store::Connection::connect()
        .await
        .expect("Could not connect to session store");
    let state = state::AppState {
        db_conn,
        session_store_conn,
    };
    let app = axum::Router::new()
        .route("/", get(root))
        .nest("/auth", routes::auth::create_router(&state))
        .nest("/onboard", routes::registration::create_router(&state))
        .with_state(state);
    let listener = TcpListener::bind("0.0.0.0:80")
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
