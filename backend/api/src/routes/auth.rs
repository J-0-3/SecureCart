//! Routes under /auth handling authentication related mechanisms.
use crate::{
    controllers::auth,
    middleware::auth::{
        authenticated_middleware, partially_authenticated_middleware, PartialUserId, UserId,
    },
    state::AppState,
};
use axum::{
    extract::{Extension, Json, State},
    handler::Handler as _,
    http::StatusCode,
    middleware::from_fn_with_state,
    routing::{get, post},
    Router,
};
use axum_extra::extract::{cookie::Cookie, CookieJar};
use serde::{Deserialize, Serialize};

/// Create a router for the /auth route.
pub fn create_router(state: &AppState) -> Router<AppState> {
    let mfa_router = Router::new()
        .route("/", get(get_mfa_methods))
        .layer(from_fn_with_state(
            state.clone(),
            partially_authenticated_middleware,
        ));
    Router::new()
        .route("/", get(root))
        .route("/methods", get(list_methods))
        .route("/login", post(login))
        .route(
            "/whoami",
            get(whoami.layer(from_fn_with_state(state.clone(), authenticated_middleware))),
        )
        .nest("/2fa", mfa_router)
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

async fn login(
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

#[derive(Serialize)]
/// A response to /auth/whoami
struct WhoamiResponse {
    /// The requesting user's ID.
    pub user_id: u64,
}
/// Get the currently authenticated user.
async fn whoami(Extension(UserId(user_id)): Extension<UserId>) -> Json<WhoamiResponse> {
    Json(WhoamiResponse { user_id })
}

#[derive(Serialize)]
/// A response to /auth/2fa
struct MfaMethodsResponse {
    /// The 2fa methods available to the user.
    methods: Vec<auth::MfaAuthenticationMethod>
}

/// Get MFA methods available to a user.
async fn get_mfa_methods(
    State(state): State<AppState>, Extension(PartialUserId(user_id)): Extension<PartialUserId>,
) -> Result<Json<MfaMethodsResponse>, StatusCode> {
    let db_conn = state.db_conn;
    let methods = auth::list_mfa_methods(user_id, &db_conn).await.map_err(|err| {
        eprintln!("SQLx error while getting MFA methods: {err}");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    
    Ok(Json(MfaMethodsResponse {
        methods
    }))
}
