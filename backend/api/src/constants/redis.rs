//! Redis connection related constants.
use std::env::var;
use std::sync::LazyLock;

/// The hostname where the Redis session store can be found.
pub static REDIS_HOST: LazyLock<String> =
    LazyLock::new(|| var("REDIS_HOST").expect("REDIS_HOST not provided in environment variables"));

/// The formatted URL which can be used to connect to Redis.
pub static REDIS_URL: LazyLock<String> = LazyLock::new(|| format!("redis://{}/", *REDIS_HOST));
