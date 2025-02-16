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
