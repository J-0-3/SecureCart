use std::{env::var, sync::LazyLock};

use super::secrets::read_secret;

pub static STRIPE_SECRET_KEY: LazyLock<String> = LazyLock::new(|| {
    var("STRIPE_SECRET_KEY").unwrap_or_else(|_| {
        let secret_path = var("STRIPE_SECRET_KEY_DOCKER_SECRET").expect(
            "Neither STRIPE_SECRET_KEY nor STRIPE_SECRET_KEY_DOCKER_SECRET provided in environment variables"
        );
        read_secret(&secret_path).expect("Failed to read STRIPE_SECRET_KEY docker secret")
    })
});

pub static STRIPE_WEBHOOK_SECRET: LazyLock<String> = LazyLock::new(|| {
    var("STRIPE_WEBHOOK_SECRET").unwrap_or_else(|_| {
        let secret_path = var("STRIPE_WEBHOOK_SECRET_DOCKER_SECRET").expect(
            "Neither STRIPE_WEBHOOK_SECRET nor STRIPE_WEBHOOK_SECRET_DOCKER_SECRET provided in environment variables"
        );
        read_secret(&secret_path).expect("Failed to read STRIPE_WEBHOOK_SECRET docker secret")
    })
});

pub static STRIPE_PUBLISHABLE_KEY: LazyLock<String> = LazyLock::new(|| {
    var("STRIPE_PUBLISHABLE_KEY").expect("STRIPE_PUBLISHABLE_KEY not set in environment variables.")
});
