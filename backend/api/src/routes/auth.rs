//! Routes under /auth handling authentication related mechanisms.
use crate::controllers::auth;
use axum::{extract::Json, routing::get, Router};
use serde::Serialize;

/// Create a router for the /auth route.
pub fn create_router() -> Router {
    Router::new()
        .route("/", get(root))
        .route("/methods", get(list_methods))
}

/// Simply returns a happy message :)
async fn root() -> Json<String> {
    Json("Authentication service is running! Yippee!".to_owned())
}

#[derive(Serialize)]
struct GetMethodsResponse {
    pub methods: Vec<auth::PrimaryAuthenticationMethod>,
}

async fn list_methods() -> Json<GetMethodsResponse> {
    Json(GetMethodsResponse {
        methods: auth::list_supported_authentication_methods(),
    })
}
