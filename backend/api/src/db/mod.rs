pub mod models;
use crate::constants::db as constants;

pub async fn connect() -> Result<sqlx::PgPool, sqlx::Error> {
    sqlx::PgPool::connect(&constants::DB_URL).await
}
