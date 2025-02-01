//! Middleware used for checking user authentication/authorisation.
use crate::{services::sessions::Session, state::AppState};
use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::Response,
};
use axum_extra::extract::CookieJar;

#[derive(Copy, Clone)]
/// A fully authenticated user ID.
pub struct UserId(pub u64);

#[derive(Copy, Clone)]
/// A partially authenticated user ID.
pub struct PartialUserId(pub u64);

/// Middleware to parse a SESSION cookie and identify the associated user.
pub async fn session_middleware(
    State(state): State<AppState>,
    cookie_jar: CookieJar,
    mut req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let session_cookie = cookie_jar
        .get("SESSION")
        .ok_or(StatusCode::UNAUTHORIZED)?
        .value();
    let mut session_store = state.session_store_conn.clone();
    let session = Session::get(session_cookie, &mut session_store)
        .await
        .map_err(|err| {
            eprintln!("Error loading session from store: {err}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or_else(|| {
            eprintln!("Invalid session token");
            StatusCode::UNAUTHORIZED
        })?;
    req.extensions_mut().insert(session);
    Ok(next.run(req).await)
}
