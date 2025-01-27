use std::fs::File;
use std::io::Read;
use std::path::Path;

const DOCKER_SECRETS_PATH: &str = "/run/secrets/";

pub fn read_secret(name: &str) -> Result<String, std::io::Error> {
    let mut secret_val = Default::default();
    File::open(Path::new(DOCKER_SECRETS_PATH).join(name.to_lowercase()))?
        .read_to_string(&mut secret_val)?;
    Ok(secret_val)
}
