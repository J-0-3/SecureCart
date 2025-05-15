use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    middleware::from_fn_with_state,
    routing::{delete, get, post, put},
    Extension, Json, Router,
};
use axum_extra::extract::{cookie::Cookie, CookieJar};
use base64::{prelude::BASE64_STANDARD, Engine as _};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    constants::passwords::{PASSWORD_MAX_LENGTH, PASSWORD_MIN_LENGTH},
    db::models::appuser::{AppUser, AppUserRole, AppUserSearchParameters},
    middleware::session::session_middleware,
    services::{
        registration,
        sessions::{AdministratorSession, GenericAuthenticatedSession},
        users,
    },
    state::AppState,
    utils::httperror::HttpError,
};

pub fn create_router(state: &AppState) -> Router<AppState> {
    let authenticated = Router::new()
        .route("/self", get(retrieve_self))
        .route("/self", put(update_self))
        .route("/self/credential", put(update_credential))
        .route("/self/2fa/new", get(generate_2fa))
        .route("/self/2fa", post(set_2fa))
        .route("/self", delete(delete_self))
        .layer(from_fn_with_state(
            state.clone(),
            session_middleware::<GenericAuthenticatedSession>,
        ));
    let administrator = Router::new()
        .route("/", get(search_users))
        .route("/{user_id}", get(retrieve_user))
        .route("/{user_id}", put(update_user))
        .route("/{user_id}", delete(delete_user))
        .route("/{user_id}/promote", post(promote_user))
        .layer(from_fn_with_state(
            state.clone(),
            session_middleware::<AdministratorSession>,
        ));
    authenticated.merge(administrator)
}

async fn retrieve_user(
    State(state): State<AppState>,
    Extension(session): Extension<AdministratorSession>,
    Path(user_id): Path<Uuid>,
) -> Result<Json<AppUser>, HttpError> {
    let user = users::retrieve_user(user_id, &state.db)
        .await?
        .ok_or_else(|| {
            eprintln!(
                "Administrator {} attempted to retrieve data of user {}, who does not exist",
                session.user_id(),
                user_id
            );
            StatusCode::NOT_FOUND
        })?;
    Ok(Json(user))
}

async fn retrieve_self(
    State(state): State<AppState>,
    Extension(session): Extension<GenericAuthenticatedSession>,
) -> Result<Json<AppUser>, HttpError> {
    Ok(Json(
        users::retrieve_user(session.user_id(), &state.db).await?.ok_or_else(|| {
            eprintln!("User {} was not found while requesting their own data. Something is critically wrong.", session.user_id());
            StatusCode::INTERNAL_SERVER_ERROR
        })?,
    ))
}

#[derive(Serialize)]
struct Generate2faResponse {
    qr: String,
    secret: String,
}

async fn generate_2fa() -> Result<Json<Generate2faResponse>, HttpError> {
    let totp = users::generate_2fa()?;
    let qr = totp.get_qr_base64().map_err(|err| {
        eprintln!("Error generating 2fa QR code: {err}");
        HttpError::new(StatusCode::INTERNAL_SERVER_ERROR, Some(err))
    })?;
    let secret = BASE64_STANDARD.encode(totp.secret);
    Ok(Json(Generate2faResponse { qr, secret }))
}

#[derive(Deserialize)]
struct Set2faRequest {
    secret: String,
    code: String,
}
async fn set_2fa(
    State(state): State<AppState>,
    Extension(session): Extension<GenericAuthenticatedSession>,
    Json(body): Json<Set2faRequest>,
) -> Result<(), HttpError> {
    let secret_raw = BASE64_STANDARD.decode(body.secret).map_err(|_err| {
        eprintln!("Invalid base64 in 2fa secret");
        HttpError::new(
            StatusCode::UNPROCESSABLE_ENTITY,
            Some(String::from("Invalid base64 encoding in 2FA secret")),
        )
    })?;
    Ok(
        users::set_2fa(session.user_id(), secret_raw, &body.code, &state.db)
            .await
            .map(|_| ())?,
    )
}
async fn update_self(
    State(state): State<AppState>,
    Extension(session): Extension<GenericAuthenticatedSession>,
    Json(body): Json<users::AppUserUpdate>,
) -> Result<Json<AppUser>, HttpError> {
    eprintln!("User {} updated their data: {}", session.user_id(), body);
    Ok(Json(
        users::update_user(session.user_id(), body, &state.db).await?,
    ))
}

async fn update_user(
    State(state): State<AppState>,
    Extension(session): Extension<AdministratorSession>,
    Path(user_id): Path<Uuid>,
    Json(body): Json<users::AppUserUpdate>,
) -> Result<Json<AppUser>, HttpError> {
    let user = AppUser::select_one(user_id, &state.db)
        .await?
        .ok_or_else(|| {
            eprintln!(
                "Administrator {} attempted to update data for user {}, who does not exist.",
                session.user_id(),
                user_id
            );
            HttpError::new(
                StatusCode::NOT_FOUND,
                Some(format!("User {} not found", session.user_id())),
            )
        })?;
    if user_id != session.user_id() && user.role == AppUserRole::Administrator {
        eprintln!(
            "Administrator {} made an unauthorised attempt to update data for administrator {}",
            session.user_id(),
            user_id
        );
        return Err(HttpError::new(
            StatusCode::FORBIDDEN,
            Some(String::from("Cannot update data for another administrator")),
        ));
    }
    eprintln!(
        "Administrator {} updated data for customer {}: {}",
        session.user_id(),
        user_id,
        body
    );
    Ok(Json(users::update_user(user_id, body, &state.db).await?))
}

#[derive(Serialize)]
struct UserSearchResponse {
    users: Vec<AppUser>,
}

async fn search_users(
    State(state): State<AppState>,
    Query(params): Query<AppUserSearchParameters>,
) -> Result<Json<UserSearchResponse>, HttpError> {
    Ok(Json(UserSearchResponse {
        users: users::search_users(params, &state.db).await?,
    }))
}

async fn promote_user(
    State(state): State<AppState>,
    Path(user_id): Path<Uuid>,
) -> Result<Json<AppUser>, HttpError> {
    eprintln!("User {user_id} is being promoted to Administrator");
    Ok(Json(users::promote_user(user_id, &state.db).await?))
}

async fn delete_user(
    cookies: CookieJar,
    State(state): State<AppState>,
    Extension(session): Extension<AdministratorSession>,
    Path(user_id): Path<Uuid>,
) -> Result<CookieJar, HttpError> {
    if user_id == session.user_id()
        && AppUser::search(
            AppUserSearchParameters {
                role: Some(AppUserRole::Administrator),
                email: None,
            },
            &state.db,
        )
        .await?
        .len()
            == 1
    {
        eprintln!("Sole administrator {user_id} attempted to delete their account. Denied until another administrator is promoted.");
        return Err(HttpError::new(
            StatusCode::FORBIDDEN,
            Some(String::from(
                "Cannot delete account until another administrator is promoted.",
            )),
        ));
    }
    if AppUser::select_one(user_id, &state.db)
        .await?
        .ok_or_else(|| {
            eprintln!(
                "Administrator {} attempted to delete user {}, who does not exist",
                session.user_id(),
                user_id
            );
            HttpError::new(
                StatusCode::NOT_FOUND,
                Some(format!("User {user_id} not found")),
            )
        })?
        .role
        == AppUserRole::Administrator
    {
        eprintln!(
            "Administrator {} attempted to delete account of administrator {}",
            session.user_id(),
            user_id
        );
        return Err(HttpError::new(
            StatusCode::FORBIDDEN,
            Some(String::from(
                "Cannot delete account of another administrator",
            )),
        ));
    }
    users::delete_user(user_id, &state.db).await?;
    if user_id == session.user_id() {
        Ok(cookies
            .remove(Cookie::from("session"))
            .remove(Cookie::from("session_csrf")))
    } else {
        eprintln!(
            "Customer {} account deleted by administrator {}",
            user_id,
            session.user_id()
        );
        Ok(cookies)
    }
}

async fn delete_self(
    cookies: CookieJar,
    State(state): State<AppState>,
    Extension(session): Extension<GenericAuthenticatedSession>,
) -> Result<CookieJar, HttpError> {
    if let GenericAuthenticatedSession::Administrator(_) = session {
        if AppUser::search(
            AppUserSearchParameters {
                role: Some(AppUserRole::Administrator),
                email: None,
            },
            &state.db,
        )
        .await?
        .len()
            == 1
        {
            eprintln!("Sole administrator {} attempted to delete their account. Denied until another administrator is promoted.", session.user_id());
            return Err(HttpError::new(
                StatusCode::FORBIDDEN,
                Some(String::from(
                    "Cannot delete account until another administrator is promoted.",
                )),
            ));
        }
    }
    users::delete_user(session.user_id(), &state.db).await?;
    eprintln!("User {} deleted their account", session.user_id());
    Ok(cookies
        .remove(Cookie::from("session"))
        .remove(Cookie::from("session_csrf")))
}

async fn update_credential(
    State(state): State<AppState>,
    Extension(session): Extension<GenericAuthenticatedSession>,
    Json(body): Json<registration::PrimaryAuthenticationMethod>,
) -> Result<(), HttpError> {
    users::update_credential(session.user_id(), body, &state.db).await?;
    eprintln!(
        "User {} has updated their primary authentication mechanism.",
        session.user_id()
    );
    Ok(())
}

impl From<users::errors::CredentialUpdateError> for HttpError {
    fn from(error: users::errors::CredentialUpdateError) -> Self {
        match error {
            users::errors::CredentialUpdateError::DatabaseError(err) => err.into(),
            users::errors::CredentialUpdateError::PasswordTooShort(user_id) => {
                eprintln!("User {user_id} attempted to update their password to below the minimum length.");
                Self::new(
                    StatusCode::UNPROCESSABLE_ENTITY,
                    Some(format!(
                        "Password is below the minimum length of {PASSWORD_MIN_LENGTH}"
                    )),
                )
            }
            users::errors::CredentialUpdateError::PasswordTooLong(user_id) => {
                eprintln!("User {user_id} attempted to update their password to above the maximum length.");
                Self::new(
                    StatusCode::UNPROCESSABLE_ENTITY,
                    Some(format!(
                        "Password is above the maximum length of {PASSWORD_MAX_LENGTH}"
                    )),
                )
            }
        }
    }
}

impl From<users::errors::UserRetrievalError> for HttpError {
    fn from(error: users::errors::UserRetrievalError) -> Self {
        match error {
            users::errors::UserRetrievalError::DatabaseError(err) => err.into(),
        }
    }
}

impl From<users::errors::UserSearchError> for HttpError {
    fn from(error: users::errors::UserSearchError) -> Self {
        match error {
            users::errors::UserSearchError::DatabaseError(err) => err.into(),
        }
    }
}

impl From<users::errors::UserPromotionError> for HttpError {
    fn from(error: users::errors::UserPromotionError) -> Self {
        match error {
            users::errors::UserPromotionError::DatabaseError(err) => err.into(),
            users::errors::UserPromotionError::UserNonExistent(user_id) => {
                eprintln!("Attempted to promote non-existent user {user_id}");
                Self::new(
                    StatusCode::NOT_FOUND,
                    Some(format!("User {user_id} not found")),
                )
            }
            users::errors::UserPromotionError::AlreadyAdministrator(user_id) => {
                eprintln!("Attempted to promote user {user_id}, who is already an administrator");
                Self::new(
                    StatusCode::CONFLICT,
                    Some(String::from("User is already an administrator")),
                )
            }
        }
    }
}

impl From<users::errors::UserDeletionError> for HttpError {
    fn from(error: users::errors::UserDeletionError) -> Self {
        match error {
            users::errors::UserDeletionError::UserNonExistent(user_id) => {
                eprintln!("Attempted to delete non-existent user {user_id}");
                Self::new(
                    StatusCode::NOT_FOUND,
                    Some(format!("User {user_id} not found")),
                )
            }
            users::errors::UserDeletionError::DatabaseError(err) => err.into(),
        }
    }
}

impl From<users::errors::UserUpdateError> for HttpError {
    fn from(error: users::errors::UserUpdateError) -> Self {
        match error {
            users::errors::UserUpdateError::UserNonExistent(user_id) => {
                eprintln!("Attempted to update non-existent user {user_id}");
                Self::new(
                    StatusCode::NOT_FOUND,
                    Some(format!("User {user_id} not found")),
                )
            }
            users::errors::UserUpdateError::DatabaseError(err) => err.into(),
        }
    }
}

impl From<users::errors::SetTotpError> for HttpError {
    fn from(error: users::errors::SetTotpError) -> Self {
        match error {
            users::errors::SetTotpError::DatabaseError(err) => err.into(),
            users::errors::SetTotpError::IncorrectCode(user_id) => {
                eprintln!("User {user_id} supplied incorrect code during 2fa setup");
                Self::new(
                    StatusCode::FORBIDDEN,
                    Some(String::from("2FA verification code incorrect")),
                )
            }
        }
    }
}

impl From<users::errors::GenerateTotpError> for HttpError {
    fn from(error: users::errors::GenerateTotpError) -> Self {
        match error {
            users::errors::GenerateTotpError::Rfc6238Error(err) => {
                eprintln!("Non-RFC6238-compliant parameters in 2fa generation: {err}");
                StatusCode::INTERNAL_SERVER_ERROR.into()
            }
        }
    }
}
