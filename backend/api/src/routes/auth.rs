//! Routes under /auth handling authentication related mechanisms.
use crate::{
    middleware::session::{session_middleware, session_middleware_no_csrf},
    services::{
        auth,
        sessions::{
            self, AdministratorSession, CustomerSession, GenericAuthenticatedSession,
            PreAuthenticationSession, SessionTrait as _,
        },
    },
    state::AppState,
    utils::{email::EmailAddress, httperror::HttpError},
};
use axum::{
    extract::{Extension, Json, State},
    http::{HeaderMap, StatusCode},
    middleware::from_fn_with_state,
    routing::{delete, get, post},
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
        .route("/", get(list_methods))
        .route("/", post(login));
    let authenticated = Router::new()
        .route("/", delete(logout))
        .layer(from_fn_with_state(
            state.clone(),
            session_middleware::<GenericAuthenticatedSession>,
        ));
    let authenticated_no_csrf =
        Router::new()
            .route("/check", get(|| async {}))
            .layer(from_fn_with_state(
                state.clone(),
                session_middleware_no_csrf::<GenericAuthenticatedSession>,
            ));
    let customer_authenticated_no_csrf = Router::new()
        .route("/check/customer", get(|| async {}))
        .layer(from_fn_with_state(
            state.clone(),
            session_middleware_no_csrf::<CustomerSession>,
        ));
    let admin_authenticated_no_csrf =
        Router::new()
            .route("/check/admin", get(|| async {}))
            .layer(from_fn_with_state(
                state.clone(),
                session_middleware_no_csrf::<AdministratorSession>,
            ));
    let pre_authenticated = Router::new()
        .route("/2fa", get(get_mfa_methods))
        .route("/2fa", post(authenticate_2fa))
        .layer(from_fn_with_state(
            state.clone(),
            session_middleware::<PreAuthenticationSession>,
        ));

    unauthenticated
        .merge(pre_authenticated)
        .merge(authenticated_no_csrf)
        .merge(authenticated)
        .merge(customer_authenticated_no_csrf)
        .merge(admin_authenticated_no_csrf)
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
    pub email: EmailAddress,
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

/// Logout the currently authenticated user.
async fn logout(
    cookies: CookieJar,
    State(state): State<AppState>,
    Extension(session): Extension<GenericAuthenticatedSession>,
) -> Result<CookieJar, HttpError> {
    session.delete(&mut state.session_store.clone()).await?;
    Ok(cookies
        .remove(Cookie::from("session"))
        .remove(Cookie::from("session_csrf")))
}

/// Login using a credential method, and set a session cookie.
async fn login(
    headers: HeaderMap,
    cookies: CookieJar,
    State(state): State<AppState>,
    Json(body): Json<AuthenticateRequest>,
) -> Result<(CookieJar, Json<AuthenticateResponse>), HttpError> {
    let client_ip = headers
        .get("x-real-ip")
        .ok_or_else(|| {
            eprintln!("X-Real-IP header not set, I should be running behind a reverse proxy.");
            HttpError::new(
                StatusCode::BAD_REQUEST,
                Some(String::from("X-Real-IP not set")),
            )
        })?
        .to_str()
        .map_err(|err| {
            eprintln!("Failed to parse X-Real-IP header value: {err}");
            HttpError::new(
                StatusCode::BAD_REQUEST,
                Some(String::from("X-Real-IP value unparseable")),
            )
        })?;
    if state
        .session_store
        .clone()
        .bruteforce_timeout(client_ip)
        .await?
    {
        eprintln!(
            "Client {client_ip} is rate-limited for suspected bruteforce authentication attempt."
        );
        return Err(HttpError::new(
            StatusCode::TOO_MANY_REQUESTS,
            Some(String::from("Too many authentication attempts.")),
        ));
    }
    let mut session_store = state.session_store.clone();
    let outcome = auth::authenticate(
        body.email.clone(),
        body.credential,
        &state.db,
        &mut session_store,
    )
    .await?;
    let (mfa_required, is_admin, token, csrf) = match outcome {
        auth::AuthenticationOutcome::Failure => {
            eprintln!("Failed authentication attempt as {}", body.email);
            return Err(HttpError::new(
                StatusCode::UNAUTHORIZED,
                Some(String::from("Authentication failed")),
            ));
        }
        auth::AuthenticationOutcome::SuccessAdministrative(session) => {
            (false, Some(true), session.token(), session.csrf_token())
        }
        auth::AuthenticationOutcome::Success(session) => {
            (false, Some(false), session.token(), session.csrf_token())
        }
        auth::AuthenticationOutcome::Partial(session) => {
            (true, None, session.token(), session.csrf_token())
        }
    };
    Ok((
        cookies
            .add(
                Cookie::build(("session", token))
                    .http_only(true)
                    .path("/")
                    .secure(true)
                    .same_site(SameSite::Strict),
            )
            .add(
                Cookie::build(("session_csrf", csrf))
                    .path("/")
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
) -> Result<Json<MfaMethodsResponse>, HttpError> {
    let db_conn = state.db;
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
) -> Result<(CookieJar, Json<MfaAuthenticateResponse>), HttpError> {
    let mut session_store = state.session_store.clone();
    let outcome =
        auth::authenticate_2fa(session, body.credential, &state.db, &mut session_store).await?;
    let (token, csrf, is_admin) = match outcome {
        auth::AuthenticationOutcome2fa::Failure => Err(HttpError::new(
            StatusCode::UNAUTHORIZED,
            Some(String::from("Two-factor authentication failed")),
        )),
        auth::AuthenticationOutcome2fa::Success(new_session) => {
            Ok((new_session.token(), new_session.csrf_token(), false))
        }
        auth::AuthenticationOutcome2fa::SuccessAdministrative(new_session) => {
            Ok((new_session.token(), new_session.csrf_token(), true))
        }
    }?;
    Ok((
        cookies
            .add(
                Cookie::build(("session", token))
                    .http_only(true)
                    .path("/")
                    .secure(true)
                    .same_site(SameSite::Strict),
            )
            .add(
                Cookie::build(("session_csrf", csrf))
                    .path("/")
                    .same_site(SameSite::Strict),
            ),
        Json(MfaAuthenticateResponse { is_admin }),
    ))
}

impl From<sessions::errors::SessionStorageError> for HttpError {
    fn from(err: sessions::errors::SessionStorageError) -> Self {
        eprintln!("Storage error while accessing session store: {err}");
        Self::new(StatusCode::INTERNAL_SERVER_ERROR, Some(err.to_string()))
    }
}
