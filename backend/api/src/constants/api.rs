//! Constants related to the general configuration of the entire API and its deployment.

use std::{env::var, sync::LazyLock};

/// A prefix to prepend to any API paths to make them externally accessible.
pub static API_URI_PREFIX: LazyLock<String> =
    LazyLock::new(|| var("API_URI_PREFIX").unwrap_or(String::from("/")));
