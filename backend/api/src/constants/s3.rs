//! S3-compatible storage related constants.
use std::{env::var, sync::LazyLock};

use super::secrets::read_secret;

pub static S3_HOST: LazyLock<String> =
    LazyLock::new(|| var("S3_HOST").expect("S3_HOST not provided in environment variables"));

pub static S3_BUCKET: LazyLock<String> =
    LazyLock::new(|| var("S3_BUCKET").expect("S3_BUCKET not provided in environment variables"));

pub static S3_ACCESS_KEY: LazyLock<String> = LazyLock::new(|| {
    var("S3_ACCESS_KEY").unwrap_or_else(|_| {
        let secret_path = var("S3_ACCESS_KEY_DOCKER_SECRET")
            .expect("Neither S3_ACCESS_KEY nor S3_ACCESS_KEY_DOCKER_SECRET provided in environment variables");
        read_secret(&secret_path).expect("Failed to read S3_ACCESS_KEY docker secret")
    })
});

pub static S3_SECRET_KEY: LazyLock<String> = LazyLock::new(|| {
    var("S3_SECRET_KEY").unwrap_or_else(|_| {
        let secret_path = var("S3_SECRET_KEY_DOCKER_SECRET").expect(
            "Neither S3_SECRET_KEY nor S3_SECRET_KEY_DOCKER_SECRET provided in environment variables",
        );
        read_secret(&secret_path).expect("Failed to read S3_SECRET_KEY docker secret")
    })
});

pub static S3_EXTERNAL_URI: LazyLock<String> =
    LazyLock::new(|| var("S3_EXTERNAL_URI").unwrap_or_else(|_| String::new()));
