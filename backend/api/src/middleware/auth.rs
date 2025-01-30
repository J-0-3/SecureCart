//! Middleware used for checking user authentication/authorisation.
use crate::{controllers::auth::SessionToken, state::AppState};
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
pub async fn authenticated_middleware(
    State(state): State<AppState>,
    cookie_jar: CookieJar,
    mut req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let session_cookie = cookie_jar
        .get("SESSION")
        .ok_or(StatusCode::UNAUTHORIZED)?
        .value();
    let token = SessionToken::Full(session_cookie.to_owned());
    let mut redis_conn = state.redis_conn.clone();
    let user_id = token
        .user_id(&mut redis_conn)
        .await
        .map_err(|err| {
            eprintln!("Error fetching user ID from Redis: {err}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or(StatusCode::UNAUTHORIZED)?;
    req.extensions_mut().insert(UserId(user_id));
    Ok(next.run(req).await)
}

/// Middleware to parse a SESSION cookie and identify the partially authenticated associated user.
pub async fn partially_authenticated_middleware(
    State(state): State<AppState>,
    cookie_jar: CookieJar,
    mut req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let session_cookie = cookie_jar
        .get("SESSION")
        .ok_or(StatusCode::UNAUTHORIZED)?
        .value();
    let token = SessionToken::Partial(session_cookie.to_owned());
    let mut redis_conn = state.redis_conn.clone();
    let user_id = token
        .user_id(&mut redis_conn)
        .await
        .map_err(|err| {
            eprintln!("Error fetching user ID from Redis: {err}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or(StatusCode::UNAUTHORIZED)?;
    req.extensions_mut().insert(PartialUserId(user_id));
    Ok(next.run(req).await)
}
