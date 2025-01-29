//! Routes under /auth handling authentication related mechanisms.
use crate::{controllers::auth, state::AppState};
use axum::{
    extract::{Json, State},
    http::StatusCode,
    routing::{get, post},
    Router,
};
use axum_extra::extract::{cookie::Cookie, CookieJar};
use serde::{Deserialize, Serialize};

/// Create a router for the /auth route.
pub fn create_router() -> Router<AppState> {
    Router::new()
        .route("/", get(root))
        .route("/methods", get(list_methods))
        .route("/login", post(authenticate))
}

/// Simply returns a happy message :)
async fn root() -> Json<String> {
    Json("Authentication service is running! Yippee!".to_owned())
}

#[derive(Serialize)]
/// The response model for /auth/methods.
struct GetMethodsResponse {
    /// The supported primary authentication methods.
    pub methods: Vec<auth::PrimaryAuthenticationMethod>,
}

/// List available primary authentication methods.
async fn list_methods() -> Json<GetMethodsResponse> {
    Json(GetMethodsResponse {
        methods: auth::list_supported_authentication_methods(),
    })
}

#[derive(Deserialize)]
/// A request to /auth/login.
struct AuthenticateRequest {
    /// The email provided at login.
    pub email: String,
    /// The credential provided at login.
    pub credential: auth::PrimaryAuthenticationMethod,
}
#[derive(Serialize)]
/// A response to /auth/login
struct AuthenticateResponse {
    /// Whether further authentication is required.
    pub mfa_required: bool,
}

async fn authenticate(
    cookies: CookieJar,
    State(state): State<AppState>,
    Json(body): Json<AuthenticateRequest>,
) -> Result<(CookieJar, Json<AuthenticateResponse>), StatusCode> {
    let session = auth::authenticate(
        &body.email,
        body.credential,
        &state.db_conn,
        &mut state.redis_conn.clone(),
    )
    .await
    .map_err(|err| {
        eprintln!("SQLx error while authenticating: {err}.");
        StatusCode::INTERNAL_SERVER_ERROR
    })?
    .ok_or_else(|| {
        eprintln!("Failed authentication as {}.", body.email);
        StatusCode::UNAUTHORIZED
    })?;
    let (mfa_required, session_cookie) = match session {
        auth::SessionToken::Full(inner) => (false, inner),
        auth::SessionToken::Partial(inner) => (true, inner),
    };
    Ok((
        cookies.add(Cookie::build(("SESSION", session_cookie)).http_only(true)),
        Json(AuthenticateResponse { mfa_required }),
    ))
}
