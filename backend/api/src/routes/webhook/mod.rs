//! Webhook API endpoints, primarily used for handling stripe events
use axum::Router;

use crate::state::AppState;

#[cfg(feature = "stripe")]
mod stripe;

/// Creates a router for all webhook interfaces.
#[cfg_attr(
    not(feature = "stripe"),
    expect(
        unused_variables,
        reason = "state will be unused when no features are enabled."
    )
)]
pub fn create_router(state: &AppState) -> Router<AppState> {
    #[cfg_attr(
        not(feature = "stripe"),
        expect(unused_mut, reason = "Only mutated when webhook features are enabled.")
    )]
    let mut router = Router::new();
    #[cfg(feature = "stripe")]
    {
        router = router.nest("/stripe", stripe::create_router());
    };
    router
}
