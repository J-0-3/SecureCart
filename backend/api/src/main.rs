//! This crate implements the backend API for the `SecureCart` ecommerce platform.

mod constants;
mod db;
mod middleware;
mod routes;
mod services;
mod state;
mod utils;

#[expect(
    clippy::useless_attribute,
    reason = "Lint is enabled only in clippy::restrictions"
)]
#[expect(
    clippy::std_instead_of_alloc,
    reason = "Does not work outside of no_std"
)]
use std::sync::Arc;

use axum::{extract::Json, routing::get};
use object_store::aws::AmazonS3Builder;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() {
    let s3 = AmazonS3Builder::new()
        .with_endpoint(format!(
            "http://{}:{}",
            &*constants::s3::S3_HOST,
            &*constants::s3::S3_PORT
        ))
        .with_bucket_name(&*constants::s3::S3_BUCKET)
        .with_access_key_id(&*constants::s3::S3_ACCESS_KEY)
        .with_secret_access_key(&*constants::s3::S3_SECRET_KEY)
        .with_allow_http(true)
        .build()
        .expect("Could not connect to S3-compatible object storage");
    println!("CONNECTED TO S3: {s3}");
    let db_conn = db::connect()
        .await
        .expect("Could not connect to primary database");
    let session_store_conn = services::sessions::store::Connection::connect()
        .await
        .expect("Could not connect to session store");
    let state = state::AppState {
        db: db_conn,
        session_store: session_store_conn,
        media_store: Arc::new(s3),
    };
    let app = axum::Router::new()
        .route("/", get(root))
        .nest("/auth", routes::auth::create_router(&state))
        .nest("/registration", routes::registration::create_router(&state))
        .nest("/products", routes::products::create_router(&state))
        .nest("/orders", routes::orders::create_router(&state))
        .nest("/webhook", routes::webhook::create_router(&state))
        .nest("/checkout", routes::checkout::create_router(&state))
        .nest("/users", routes::users::create_router(&state))
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
