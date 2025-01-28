//! Redis connection related constants.
use std::sync::LazyLock;
use std::env::var;

pub static REDIS_HOST: LazyLock<String> = LazyLock::new(|| {
    var("REDIS_HOST").expect("REDIS_HOST not provided in environment variables")
});
