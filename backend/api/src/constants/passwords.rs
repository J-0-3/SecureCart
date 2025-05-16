//! Constants for configuring the application's password policy

/// The minimum password length users can set.
pub const PASSWORD_MIN_LENGTH: usize = 8;
/// The maximum password length users can set (to avoid Argon2 DOS).
pub const PASSWORD_MAX_LENGTH: usize = 128;
