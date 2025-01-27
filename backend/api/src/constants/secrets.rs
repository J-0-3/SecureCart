//! Utilities for handling docker secrets.
use std::fs::File;
use std::io::{Error, Read as _};
use std::path::Path;

/// The path at which Docker mounted secrets are stored.
const DOCKER_SECRETS_PATH: &str = "/run/secrets/";

/// Attempts to read a Docker mounted secret from the filesystem.
pub fn read_secret(name: &str) -> Result<String, Error> {
    let mut secret_val = String::default();
    File::open(Path::new(DOCKER_SECRETS_PATH).join(name.to_lowercase()))?
        .read_to_string(&mut secret_val)?;
    Ok(secret_val)
}
