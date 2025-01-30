//! This crate implements the backend API for the `SecureCart` ecommerce platform.

mod constants;
mod controllers;
mod db;
mod routes;
mod state;
mod utils;
mod middleware;

use axum::{extract::Json, routing::get};
use tokio::net::TcpListener;

#[tokio::main]
async fn main() {
    let db_conn = db::connect()
        .await
        .expect("Could not connect to to Postgres");
    let redis_conn = redis::Client::open(constants::redis::REDIS_URL.clone())
        .expect("Could not connect to Redis")
        .get_multiplexed_async_connection()
        .await
        .expect("Could not get async Redis connection");
    let state = state::AppState {
        db_conn,
        redis_conn,
    };
    let app = axum::Router::new()
        .route("/", get(root))
        .nest("/auth", routes::auth::create_router(&state))
        .with_state(state);
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
