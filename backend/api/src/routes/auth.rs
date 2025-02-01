//! Routes under /auth handling authentication related mechanisms.
use crate::{
    middleware::auth::session_middleware,
    services::{
        auth,
        sessions::{AuthenticatedFromSessionError, AuthenticatedSession, Session},
    },
    state::AppState,
};
use axum::{
    extract::{Extension, Json, State},
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
        .route("/", post(authenticate_2fa));
    Router::new()
        .route("/whoami", get(whoami))
        .nest("/2fa", mfa_router)
        .layer(from_fn_with_state(state.clone(), session_middleware))
        .route("/", get(root))
        .route("/methods", get(list_methods))
        .route("/login", post(login))
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
    let mut session_store = state.session_store_conn.clone();
    let session = auth::authenticate(
        &body.email,
        body.credential,
        &state.db_conn,
        &mut session_store,
    )
    .await
    .map_err(|err| {
        eprintln!("SQLx error while authenticating: {err:?}.");
        StatusCode::INTERNAL_SERVER_ERROR
    })?
    .ok_or_else(|| {
        eprintln!("Failed authentication as {}.", body.email);
        StatusCode::UNAUTHORIZED
    })?;
    let token = session.token().to_owned();
    let mfa_required = match AuthenticatedSession::try_from_session(session, &mut session_store)
        .await
    {
        Ok(_) => Ok(false),
        Err(err) => {
            match err {
                AuthenticatedFromSessionError::InvalidSession => {
                    eprintln!("Session expired immediately after creation. Server is likely misconfigured.");
                    Err(StatusCode::INTERNAL_SERVER_ERROR) // not a 401, this is probably on us
                }
                AuthenticatedFromSessionError::StorageError(error) => {
                    eprintln!("Storage error while checking session authentication: {error:?}");
                    Err(StatusCode::INTERNAL_SERVER_ERROR)
                }
                AuthenticatedFromSessionError::NotAuthenticated => Ok(true),
            }
        }
    }?;
    Ok((
        cookies.add(Cookie::build(("SESSION", token)).http_only(true)),
        Json(AuthenticateResponse { mfa_required }),
    ))
}

#[derive(Serialize)]
/// A response to /auth/whoami
struct WhoamiResponse {
    /// The requesting user's ID.
    user_id: u64,
}
/// Get the currently authenticated user.
async fn whoami(Extension(session): Extension<Session>) -> Json<WhoamiResponse> {
    Json(WhoamiResponse {
        user_id: session.user_id(),
    })
}

#[derive(Serialize)]
/// A response to /auth/2fa
struct MfaMethodsResponse {
    /// The 2fa methods available to the user.
    methods: Vec<auth::MfaAuthenticationMethod>,
}

/// Get MFA methods available to a user.
async fn get_mfa_methods(
    State(state): State<AppState>,
    Extension(session): Extension<Session>,
) -> Result<Json<MfaMethodsResponse>, StatusCode> {
    let db_conn = state.db_conn;
    let methods = auth::list_mfa_methods(session.user_id(), &db_conn)
        .await
        .map_err(|err| {
            eprintln!("SQLx error while getting MFA methods: {err}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(MfaMethodsResponse { methods }))
}

#[derive(Deserialize)]
/// A request POST to /auth/2fa.
struct MfaAuthenticateRequest {
    /// The selected 2fa authentication method.
    credential: auth::MfaAuthenticationMethod,
}

/// Authenticate using an MFA method.
async fn authenticate_2fa(
    cookies: CookieJar,
    State(state): State<AppState>,
    Extension(session): Extension<Session>,
    Json(body): Json<MfaAuthenticateRequest>,
) -> Result<CookieJar, StatusCode> {
    let mut session_store = state.session_store_conn.clone();
    let user_id = session.user_id();
    let authenticated_session =
        auth::authenticate_2fa(session, body.credential, &state.db_conn, &mut session_store)
            .await
            .map_err(|err| {
                eprintln!("SQLx error while authenticating: {err:?}.");
                StatusCode::INTERNAL_SERVER_ERROR
            })?
            .ok_or_else(|| {
                eprintln!("Failed MFA authentication for user {user_id}.");
                StatusCode::UNAUTHORIZED
            })?;
    Ok(cookies
        .add(Cookie::build(("SESSION", authenticated_session.token().to_owned())).http_only(true)))
}
