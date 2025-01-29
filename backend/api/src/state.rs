use sqlx::PgPool;
use redis::aio::MultiplexedConnection;

#[derive(Clone)]
pub struct AppState {
    pub db_conn: PgPool,
    pub redis_conn: MultiplexedConnection
}
