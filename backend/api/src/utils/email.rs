//! Utilities for working with and parsing/validating email addresses.
use core::fmt;
use std::sync::LazyLock;

use serde::{de, Deserialize, Serialize};

/// Regex used to validate email address format. Non-comprehensive but good enough.
static EMAIL_REGEX: LazyLock<regex::Regex> = LazyLock::new(|| {
    regex::Regex::new(r"^[a-zA-Z0-9_.+-]+@[a-zA-Z0-9-]+(\.[a-zA-Z0-9-]+)+$")
        .expect("Email regex invalid")
});

/// A struct wrapping a `String` which is guaranteed to be a valid email address.
#[derive(Clone, sqlx::Type)]
#[sqlx(transparent)]
pub struct EmailAddress(String);

impl fmt::Display for EmailAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl TryFrom<&str> for EmailAddress {
    type Error = ();
    fn try_from(string: &str) -> Result<Self, Self::Error> {
        Self::try_from(string.to_owned())
    }
}

impl TryFrom<String> for EmailAddress {
    type Error = ();
    fn try_from(string: String) -> Result<Self, Self::Error> {
        if EMAIL_REGEX.is_match(&string) {
            Ok(Self(string))
        } else {
            Err(())
        }
    }
}

impl From<EmailAddress> for String {
    fn from(addr: EmailAddress) -> Self {
        let EmailAddress(inner) = addr;
        inner
    }
}

impl<'de> Deserialize<'de> for EmailAddress {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let str = String::deserialize(deserializer)?;
        Self::try_from(str).map_err(|_err| de::Error::custom("malformed email address"))
    }
}

impl Serialize for EmailAddress {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.0.serialize(serializer)
    }
}
