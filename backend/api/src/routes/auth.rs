//! Routes under /auth handling authentication related mechanisms.
use crate::{
    middleware::auth::session_middleware,
    services::{
        auth,
        sessions::{self, AuthenticatedSession, PreAuthenticationSession, SessionTrait as _},
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
        .route("/", post(authenticate_2fa))
        .layer(from_fn_with_state(
            state.clone(),
            session_middleware::<PreAuthenticationSession>,
        ));
    Router::new()
        .route("/whoami", get(whoami))
        .layer(from_fn_with_state(
            state.clone(),
            session_middleware::<AuthenticatedSession>,
        ))
        .nest("/2fa", mfa_router)
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
    let outcome = auth::authenticate(
        &body.email,
        body.credential,
        &state.db_conn,
        &mut session_store,
    )
    .await?;
    let (mfa_required, token) = match outcome {
        auth::AuthenticationOutcome::Failure => {
            eprintln!("Failed authentication as {}", body.email);
            return Err(StatusCode::UNAUTHORIZED);
        }
        auth::AuthenticationOutcome::Success(session) => (false, session.info().token()),
        auth::AuthenticationOutcome::Partial(session) => (true, session.info().token()),
    };
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
async fn whoami(Extension(session): Extension<AuthenticatedSession>) -> Json<WhoamiResponse> {
    Json(WhoamiResponse {
        user_id: session.info().user_id(),
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
    Extension(session): Extension<PreAuthenticationSession>,
) -> Result<Json<MfaMethodsResponse>, StatusCode> {
    let db_conn = state.db_conn;
    let methods = auth::list_mfa_methods(session.info().user_id(), &db_conn).await?;
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
    Extension(session): Extension<PreAuthenticationSession>,
    Json(body): Json<MfaAuthenticateRequest>,
) -> Result<CookieJar, StatusCode> {
    let mut session_store = state.session_store_conn.clone();
    let user_id = session.info().user_id();
    let authenticated_session =
        auth::authenticate_2fa(session, body.credential, &state.db_conn, &mut session_store)
            .await?
            .ok_or_else(|| {
                eprintln!("Failed MFA authentication for user {user_id}.");
                StatusCode::UNAUTHORIZED
            })?;
    Ok(cookies
        .add(Cookie::build(("SESSION", authenticated_session.info().token())).http_only(true)))
}

impl From<auth::errors::StorageError> for StatusCode {
    fn from(value: auth::errors::StorageError) -> Self {
        eprintln!("Storage error in route handler: {value}");
        Self::INTERNAL_SERVER_ERROR
    }
}

impl From<sessions::errors::SessionPromotionError> for StatusCode {
    fn from(value: sessions::errors::SessionPromotionError) -> Self {
        match value {
            sessions::errors::SessionPromotionError::InvalidSession => {
                eprintln!(
                    "Session expired immediately after creation. Server is likely misconfigured."
                );
                Self::INTERNAL_SERVER_ERROR // not a 401, this is probably on us
            }
            sessions::errors::SessionPromotionError::StorageError(error) => {
                eprintln!("Storage error while checking session authentication: {error}");
                Self::INTERNAL_SERVER_ERROR
            }
            sessions::errors::SessionPromotionError::NotAuthenticated => {
                eprintln!("Session is not authenticated.");
                Self::UNAUTHORIZED
            }
        }
    }
}
