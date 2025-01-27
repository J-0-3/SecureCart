//! Utilities for working with and parsing/validating email addresses.
use std::sync::LazyLock;

/// Regex used to validate email address format. Non-comprehensive but good enough.
static EMAIL_REGEX: LazyLock<regex::Regex> = LazyLock::new(|| {
    regex::Regex::new(r"^[a-zA-Z0-9_.+-]+@[a-zA-Z0-9-]+(\.[a-zA-Z0-9-]+)+$")
        .expect("Email regex invalid")
});

/// A struct wrapping a `String` which is guaranteed to be a valid email address.
#[derive(sqlx::Type)]
pub struct EmailAddress(String);

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
