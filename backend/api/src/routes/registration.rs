//! Routes for onboarding and user registration.
use crate::{
    db::models::appuser::AppUserInsert,
    middleware::auth::session_middleware,
    services::{
        registration::{self, PrimaryAuthenticationMethod},
        sessions::{RegistrationSession, SessionTrait as _},
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
use serde::Deserialize;

/// Create a router for the /onboarding route.
pub fn create_router(state: &AppState) -> Router<AppState> {
    Router::new()
        .route("/credential", post(signup_add_credential))
        .layer(from_fn_with_state(
            state.clone(),
            session_middleware::<RegistrationSession>,
        ))
        .route("/", get(root))
        .route("/signup", post(signup_init))
}

/// The root route for /onboarding, which does nothing.
async fn root() -> Json<String> {
    Json("Registration service is running!".to_owned())
}

/// Request body for /onboard/signup.
#[derive(Deserialize)]
struct SignUpInitRequest {
    /// The user data to store for the new user.
    pub user_data: AppUserInsert,
}

/// This route initialises the onboarding process by creating a temporary
/// registration session with the user's data associated with it. The database
/// will not be modified until the signup process is fully complete, and the
/// data will be deleted after the registration timeout period expires.
async fn signup_init(
    cookies: CookieJar,
    State(state): State<AppState>,
    Json(body): Json<SignUpInitRequest>,
) -> Result<CookieJar, StatusCode> {
    let mut session_store_conn = state.session_store_conn.clone();
    let db_conn = &state.db_conn;
    let session =
        registration::signup_init(body.user_data, &mut session_store_conn, db_conn).await?;
    Ok(cookies.add(Cookie::build(("SESSION", session.token())).http_only(true)))
}

/// Request body for /onboard/credential.
#[derive(Deserialize)]
struct SignUpAddCredentialRequest {
    /// The actual credential being added.
    pub credential: PrimaryAuthenticationMethod,
}

/// This route completes the onboarding process by assigning the user a credential
/// by which they can sign in. Once this is done all data will be saved to the
/// database for future logins, and the registration session will be expired.
async fn signup_add_credential(
    State(state): State<AppState>,
    Extension(session): Extension<RegistrationSession>,
    Json(body): Json<SignUpAddCredentialRequest>,
) -> Result<(), StatusCode> {
    registration::signup_add_credential_and_commit(session, body.credential, &state.db_conn)
        .await?;
    Ok(())
}

impl From<registration::errors::SignupInitError> for StatusCode {
    fn from(value: registration::errors::SignupInitError) -> Self {
        match value {
            registration::errors::SignupInitError::StorageError(err) => err.into(),
            registration::errors::SignupInitError::DuplicateEmail => {
                eprintln!("Attempt to sign up with duplicate email.");
                Self::CONFLICT
            }
        }
    }
}
