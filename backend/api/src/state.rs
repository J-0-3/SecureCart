//! Defines the state shared across the Axum application.
use redis::aio::MultiplexedConnection;
use sqlx::PgPool;

#[derive(Clone)]
/// The state struct shared across routers.
pub struct AppState {
    /// A Postgres connection pool for getting new Postgres connections.
    pub db_conn: PgPool,
    /// A redis multiplexed connection for getting new async redis connections.
    pub redis_conn: MultiplexedConnection,
}
