//! Database connection related constants.
use super::secrets::read_secret;
use std::env::var;
use std::sync::LazyLock;

/// The hostname where the database server is accessible.
pub static DB_HOST: LazyLock<String> =
    LazyLock::new(|| var("DB_HOST").expect("DB_HOST not provided in environment variables"));

/// The username to authenticate to the database server with.
pub static DB_USERNAME: LazyLock<String> = LazyLock::new(|| {
    var("DB_USERNAME").expect("DB_USERNAME not provided in environment variables")
});

/// The database to connect to on the database server.
pub static DB_DATABASE: LazyLock<String> = LazyLock::new(|| {
    var("DB_DATABASE").expect("DB_DATABASE not provided in environment variables")
});

/// The password to authenticate to the database with.
pub static DB_PASSWORD: LazyLock<String> = LazyLock::new(|| {
    var("DB_PASSWORD").unwrap_or_else(|_| {
        let secret_path = var("DB_PASSWORD_DOCKER_SECRET").expect(
            "Neither DB_PASSWORD nor DB_PASSWORD_DOCKER_SECRET provided in environment variables",
        );
        read_secret(&secret_path).expect("Failed to read DB_PASSWORD docker secret")
    })
});

/// A URL-style database connection string, provided for convenience as it simply
/// combines the other exposed constants in a defined manner.
pub static DB_URL: LazyLock<String> = LazyLock::new(|| {
    format!(
        "postgres://{}:{}@{}/{}",
        DB_USERNAME.clone(),
        DB_PASSWORD.clone(),
        DB_HOST.clone(),
        DB_DATABASE.clone()
    )
});
