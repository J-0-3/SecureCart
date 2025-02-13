//! Defines the state shared across the Axum application.
#[expect(
    clippy::useless_attribute,
    reason = "Lint is enabled only in clippy::restrictions"
)]
#[expect(
    clippy::std_instead_of_alloc,
    reason = "Does not work outside of no_std"
)]
use std::sync::Arc;

use crate::{db, services::sessions};
use object_store::ObjectStore;

#[derive(Clone)]
/// The state struct shared across routers.
pub struct AppState {
    /// A database connection pool for getting new database connections.
    pub db: db::ConnectionPool,
    /// A multiplexed connection for getting new session store connections.
    pub session_store: sessions::store::Connection,
    /// A shared connection for adding to the media store.
    pub media_store: Arc<dyn ObjectStore>,
}
