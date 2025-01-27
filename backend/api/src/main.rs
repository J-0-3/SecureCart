mod constants;
mod db;
mod routes;
mod controllers;
mod utils;

#[tokio::main]
async fn main() {
    let app = axum::Router::new()
        .route("/", axum::routing::get(root));
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080")
        .await
        .expect("Failed to bind listener");
    axum::serve(listener, app)
        .await
        .expect("Failed to init Axum service");
}

async fn root() -> String {
    "Auth service is running!".to_string()
}

