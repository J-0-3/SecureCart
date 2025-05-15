//! Routes for onboarding and user registration.
use crate::{
    constants::passwords::{PASSWORD_MAX_LENGTH, PASSWORD_MIN_LENGTH},
    db::models::appuser::AppUserInsert,
    middleware::session::session_middleware,
    services::{
        registration::{self, PrimaryAuthenticationMethod},
        sessions::{RegistrationSession, SessionTrait as _},
    },
    state::AppState,
    utils::httperror::HttpError,
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
        .route("/", post(signup_init))
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
) -> Result<CookieJar, HttpError> {
    let mut session_store_conn = state.session_store.clone();
    let db_conn = &state.db;
    let session =
        registration::signup_init(body.user_data, &mut session_store_conn, db_conn).await?;
    Ok(cookies
        .add(
            Cookie::build(("session", session.token()))
                .http_only(true)
                .path("/")
                .secure(true)
                .same_site(SameSite::Strict),
        )
        .add(
            Cookie::build(("session_csrf", session.csrf_token()))
                .path("/")
                .same_site(SameSite::Strict),
        ))
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
) -> Result<(), HttpError> {
    let mut session_store_conn = state.session_store.clone();
    registration::add_credential_and_commit(
        session,
        body.credential,
        &state.db,
        &mut session_store_conn,
    )
    .await?;
    Ok(())
}

impl From<registration::errors::SignupInitError> for HttpError {
    fn from(value: registration::errors::SignupInitError) -> Self {
        match value {
            registration::errors::SignupInitError::StorageError(err) => err.into(),
            registration::errors::SignupInitError::DuplicateEmail(email) => {
                eprintln!("Attempt to sign up with duplicate email {email}.");
                Self::new(
                    StatusCode::CONFLICT,
                    Some(format!("Email {email} is already in use.")),
                )
            }
            registration::errors::SignupInitError::EmptyAddress => {
                eprintln!("Attempt to sign up with empty address");
                Self::new(
                    StatusCode::UNPROCESSABLE_ENTITY,
                    Some(String::from("Address cannot be empty")),
                )
            }
            registration::errors::SignupInitError::EmptySurname => {
                eprintln!("Attempt to sign up with empty surname");
                Self::new(
                    StatusCode::UNPROCESSABLE_ENTITY,
                    Some(String::from("surname cannot be empty")),
                )
            }
            registration::errors::SignupInitError::EmptyForename => {
                eprintln!("Attempt to sign up with empty forename");
                Self::new(
                    StatusCode::UNPROCESSABLE_ENTITY,
                    Some(String::from("forename cannot be empty")),
                )
            }
        }
    }
}

impl From<registration::errors::AddCredentialError> for HttpError {
    fn from(value: registration::errors::AddCredentialError) -> Self {
        match value {
            registration::errors::AddCredentialError::StorageError(err) => err.into(),
            registration::errors::AddCredentialError::PasswordTooShort => {
                eprintln!("Signup attempt with password below minimum length.");
                Self::new(
                    StatusCode::UNPROCESSABLE_ENTITY,
                    Some(format!(
                        "Password is below the minimum length of {PASSWORD_MIN_LENGTH}"
                    )),
                )
            }
            registration::errors::AddCredentialError::PasswordTooLong => {
                eprintln!("Signup attempt with password above maximum length.");
                Self::new(
                    StatusCode::UNPROCESSABLE_ENTITY,
                    Some(format!(
                        "Password is above the maximum length of {PASSWORD_MAX_LENGTH}."
                    )),
                )
            }
        }
    }
}
