//! Routes under /auth handling authentication related mechanisms.
use crate::{
    middleware::auth::session_middleware,
    services::{
        auth,
        errors::StorageError,
        sessions::{
            self, AdministratorSession, CustomerSession, PreAuthenticationSession,
            SessionTrait as _,
        },
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
use axum_extra::extract::{
    cookie::{Cookie, SameSite},
    CookieJar,
};
use serde::{Deserialize, Serialize};

/// Create a router for the /auth route.
pub fn create_router(state: &AppState) -> Router<AppState> {
    let unauthenticated = Router::new()
        .route("/", get(root))
        .route("/methods", get(list_methods))
        .route("/login", post(login));
    let pre_authenticated = Router::new()
        .route("/2fa/methods", get(get_mfa_methods))
        .route("/2fa", post(authenticate_2fa))
        .layer(from_fn_with_state(
            state.clone(),
            session_middleware::<PreAuthenticationSession>,
        ));
    let customer_authenticated = Router::new()
        .route("/check/customer", get(|| async {}))
        .layer(from_fn_with_state(
            state.clone(),
            session_middleware::<CustomerSession>,
        ));
    let admin_authenticated =
        Router::new()
            .route("/check/admin", get(|| async {}))
            .layer(from_fn_with_state(
                state.clone(),
                session_middleware::<AdministratorSession>,
            ));
    unauthenticated
        .merge(pre_authenticated)
        .merge(customer_authenticated)
        .merge(admin_authenticated)
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
    /// Whether the session is administrative, None if MFA is required.
    pub is_admin: Option<bool>,
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
    let (mfa_required, is_admin, token) = match outcome {
        auth::AuthenticationOutcome::Failure => {
            eprintln!("Failed authentication as {}", body.email);
            return Err(StatusCode::UNAUTHORIZED);
        }
        auth::AuthenticationOutcome::SuccessAdministrative(session) => {
            (false, Some(true), session.token())
        }
        auth::AuthenticationOutcome::Success(session) => (false, Some(false), session.token()),
        auth::AuthenticationOutcome::Partial(session) => (true, None, session.token()),
    };
    Ok((
        cookies.add(
            Cookie::build(("SESSION", token))
                .http_only(true)
                .path("/")
                .secure(true)
                .same_site(SameSite::Strict),
        ),
        Json(AuthenticateResponse {
            mfa_required,
            is_admin,
        }),
    ))
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
    let methods = auth::list_mfa_methods(session.user_id(), &db_conn).await?;
    Ok(Json(MfaMethodsResponse { methods }))
}

#[derive(Deserialize)]
/// A request POST to /auth/2fa.
struct MfaAuthenticateRequest {
    /// The selected 2fa authentication method.
    credential: auth::MfaAuthenticationMethod,
}

#[derive(Serialize)]
struct MfaAuthenticateResponse {
    /// Whether the new session is administrative.
    is_admin: bool,
}

/// Authenticate using an MFA method.
async fn authenticate_2fa(
    cookies: CookieJar,
    State(state): State<AppState>,
    Extension(session): Extension<PreAuthenticationSession>,
    Json(body): Json<MfaAuthenticateRequest>,
) -> Result<(CookieJar, Json<MfaAuthenticateResponse>), StatusCode> {
    let mut session_store = state.session_store_conn.clone();
    let outcome =
        auth::authenticate_2fa(session, body.credential, &state.db_conn, &mut session_store)
            .await?;
    match outcome {
        auth::AuthenticationOutcome2fa::Failure => Err(StatusCode::UNAUTHORIZED),
        auth::AuthenticationOutcome2fa::Success(new_session) => Ok((
            cookies.add(
                Cookie::build(("SESSION", new_session.token()))
                    .http_only(true)
                    .path("/")
                    .secure(true)
                    .same_site(SameSite::Strict),
            ),
            Json(MfaAuthenticateResponse { is_admin: false }),
        )),
        auth::AuthenticationOutcome2fa::SuccessAdministrative(new_session) => Ok((
            cookies.add(
                Cookie::build(("SESSION", new_session.token()))
                    .http_only(true)
                    .path("/")
                    .secure(true)
                    .same_site(SameSite::Strict),
            ),
            Json(MfaAuthenticateResponse { is_admin: true }),
        )),
    }
}

impl From<StorageError> for StatusCode {
    fn from(value: StorageError) -> Self {
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
