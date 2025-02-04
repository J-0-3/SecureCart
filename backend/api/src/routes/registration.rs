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

async fn root() -> Json<String> {
    Json("Registration service is running!".to_owned())
}

#[derive(Deserialize)]
struct SignUpInitRequest {
    pub user_data: AppUserInsert,
}

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

#[derive(Deserialize)]
struct SignUpAddCredentialRequest {
    pub credential: PrimaryAuthenticationMethod,
}

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
