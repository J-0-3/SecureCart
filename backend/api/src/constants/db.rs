use super::secrets::read_secret;
use std::sync::LazyLock;

pub static DB_HOST: LazyLock<String> = LazyLock::new(|| {
    std::env::var("DB_HOST").expect("DB_HOST not provided in environment variables")
});

pub static DB_USERNAME: LazyLock<String> = LazyLock::new(|| {
    std::env::var("DB_USERNAME").expect("DB_USERNAME not provided in environment variables")
});

pub static DB_DATABASE: LazyLock<String> = LazyLock::new(|| {
    std::env::var("DB_DATABASE").expect("DB_DATABASE not provided in environment variables")
});

pub static DB_PASSWORD: LazyLock<String> = LazyLock::new(|| {
    std::env::var("DB_PASSWORD").unwrap_or_else(|_| {
        let secret_path = std::env::var("DB_PASSWORD_DOCKER_SECRET").expect(
            "Neither DB_PASSWORD nor DB_PASSWORD_DOCKER_SECRET provided in environment variables",
        );
        read_secret(&secret_path).expect("Failed to read DB_PASSWORD docker secret")
    })
});

pub static DB_URL: LazyLock<String> = LazyLock::new(|| {
    format!(
        "postgres://{}:{}@{}/{}",
        DB_USERNAME.clone(),
        DB_PASSWORD.clone(),
        DB_HOST.clone(),
        DB_DATABASE.clone()
    )
});
