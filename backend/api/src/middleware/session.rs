//! Middleware used for checking user authentication/authorisation.
use std::sync::LazyLock;

use crate::{services::sessions::SessionTrait, state::AppState};
use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::Response,
};
use axum_extra::extract::CookieJar;

/// The status code used for a CSRF failure. 419 is non-standard but
///  it's what Laravel does.
#[expect(clippy::unwrap_used, reason = "This will never panic")]
static STATUS_CODE_BAD_CSRF: LazyLock<StatusCode> =
    LazyLock::new(|| StatusCode::from_u16(419).unwrap());

/// Middleware to parse a session cookie and identify the associated user.
pub async fn session_middleware<T: SessionTrait + 'static>(
    State(state): State<AppState>,
    cookie_jar: CookieJar,
    mut req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let session_cookie = cookie_jar
        .get("session")
        .ok_or(StatusCode::UNAUTHORIZED)?
        .value();
    let session = T::get(session_cookie, &mut state.session_store.clone())
        .await
        .map_err(|err| {
            eprintln!("Error loading session from store: {err}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or_else(|| {
            eprintln!("Invalid session token.");
            StatusCode::UNAUTHORIZED
        })?;
    let csrf_token = req
        .headers()
        .get("X-CSRF-Token")
        .ok_or_else(|| {
            eprintln!("Request is missing X-CSRF-Token");
            *STATUS_CODE_BAD_CSRF
        })?
        .to_str()
        .map_err(|_err| {
            eprintln!("CSRF token contains non-ASCII.");
            StatusCode::BAD_REQUEST
        })?;
    if csrf_token != session.csrf_token() {
        eprintln!("Incorrect X-CSRF-Token in request");
        return Err(*STATUS_CODE_BAD_CSRF);
    }
    req.extensions_mut().insert(session);
    Ok(next.run(req).await)
}

/// Does the same thing as `session_middleware`, but skips the CSRF check. This
/// potentially enables CSRF attacks against the endpoint in question, so
/// ensure it does not have any dangerous effects.
pub async fn session_middleware_no_csrf<T: SessionTrait + 'static>(
    State(state): State<AppState>,
    cookie_jar: CookieJar,
    mut req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let session_cookie = cookie_jar
        .get("session")
        .ok_or(StatusCode::UNAUTHORIZED)?
        .value();
    let session = T::get(session_cookie, &mut state.session_store.clone())
        .await
        .map_err(|err| {
            eprintln!("Error loading session from store: {err}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or_else(|| {
            eprintln!("Invalid session token.");
            StatusCode::UNAUTHORIZED
        })?;
    req.extensions_mut().insert(session);
    Ok(next.run(req).await)
}
