//! S3-compatible storage related constants.
use std::{env::var, sync::LazyLock};

use super::secrets::read_secret;

/// The hostname where the S3-compatible storage service can be accessed.
pub static S3_HOST: LazyLock<String> =
    LazyLock::new(|| var("S3_HOST").expect("S3_HOST not provided in environment variables"));

/// The port where the S3-compatible storage service can be accessed.
pub static S3_PORT: LazyLock<u16> = LazyLock::new(|| {
    var("S3_PORT")
        .expect("S3_PORT not provided in environment variables")
        .parse()
        .expect("S3_PORT is not a valid port number")
});

/// The bucket where application media data is stored.
pub static S3_BUCKET: LazyLock<String> =
    LazyLock::new(|| var("S3_BUCKET").expect("S3_BUCKET not provided in environment variables"));

/// The access key (user) to authenticate to the store with.
pub static S3_ACCESS_KEY: LazyLock<String> = LazyLock::new(|| {
    var("S3_ACCESS_KEY").unwrap_or_else(|_| {
        let secret_path = var("S3_ACCESS_KEY_DOCKER_SECRET")
            .expect("Neither S3_ACCESS_KEY nor S3_ACCESS_KEY_DOCKER_SECRET provided in environment variables");
        read_secret(&secret_path).expect("Failed to read S3_ACCESS_KEY docker secret")
    })
});

/// The secret key (password) to authenticate to the store with.
pub static S3_SECRET_KEY: LazyLock<String> = LazyLock::new(|| {
    var("S3_SECRET_KEY").unwrap_or_else(|_| {
        let secret_path = var("S3_SECRET_KEY_DOCKER_SECRET").expect(
            "Neither S3_SECRET_KEY nor S3_SECRET_KEY_DOCKER_SECRET provided in environment variables",
        );
        read_secret(&secret_path).expect("Failed to read S3_SECRET_KEY docker secret")
    })
});

/// An optional URI where the S3 storage can be accessed from outside the
/// inter-service internal network. Can be left blank, and the store will be
/// assumed to be accessible via the same host as the API (true in the default
/// docker compose configuration with NGINX).
pub static S3_EXTERNAL_URI: LazyLock<String> =
    LazyLock::new(|| var("S3_EXTERNAL_URI").unwrap_or_else(|_| String::new()));
