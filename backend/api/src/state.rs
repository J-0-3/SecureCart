//! Defines the state shared across the Axum application.
use crate::{db, services::sessions};

#[derive(Clone)]
/// The state struct shared across routers.
pub struct AppState {
    /// A database connection pool for getting new database connections.
    pub db_conn: db::ConnectionPool,
    /// A multiplexed connection for getting new session store connections.
    pub session_store_conn: sessions::store::Connection,
}
