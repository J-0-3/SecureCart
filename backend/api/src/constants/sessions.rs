//! Constants related to authentication and session handling.

/// Timeout for authenticated sessions in seconds.
pub const SESSION_TIMEOUT: u32 = 7 * 24 * 60 * 60;
/// Timeout for pre-authentication sessions in seconds.
pub const PREAUTH_SESSION_TIMEOUT: u32 = 5 * 60;
/// Timeout for registration sessions in seconds;
pub const REGISTRATION_SESSION_TIMEOUT: u32 = 10 * 60;
/// Timeout for administrative sessions in seconds.
pub const ADMIN_SESSION_TIMEOUT: u32 = 2 * 60 * 60;
/// Max authentication attempts before timeout;
pub const AUTH_TIMEOUT_ATTEMPTS: u32 = 5;
/// The timeout period within which bruteforce auth attempts will be counted;
pub const AUTH_TIMEOUT_PERIOD: u32 = 10;
/// The period for which a client is timed out after being flagged for bruteforce
pub const AUTH_PENALTY_PERIOD: u32 = 60;
