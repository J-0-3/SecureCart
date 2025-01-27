use std::sync::LazyLock;

static EMAIL_REGEX: LazyLock<regex::Regex> = LazyLock::new(|| {
    regex::Regex::new(r"^[a-zA-Z0-9_.+-]+@[a-zA-Z0-9-]+(\.[a-zA-Z0-9-]+)+$")
        .expect("Email regex invalid")
});

#[derive(sqlx::Type)]
pub struct EmailAddress(pub String);

impl TryFrom<&str> for EmailAddress {
    type Error = ();
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        Self::try_from(s.to_string())
    }
}

impl TryFrom<String> for EmailAddress {
    type Error = ();
    fn try_from(s: String) -> Result<Self, Self::Error> {
        if EMAIL_REGEX.is_match(&s) {
            Ok(Self(s))
        } else {
            Err(())
        }
    }
}

impl From<EmailAddress> for String {
    fn from(addr: EmailAddress) -> Self {
        let EmailAddress(s) = addr;
        s
    }
}

