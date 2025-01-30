//! Controllers which manage authentication.
use crate::{
    constants::sessions::{AUTH_SESSION_TIMEOUT, SESSION_TIMEOUT},
    db::models::{
        appuser::AppUser,
        password::Password,
        totp::{self, Totp},
    },
};
use rand::{rngs::StdRng, Rng as _, SeedableRng as _};
use redis::{aio::MultiplexedConnection, AsyncCommands as _};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

#[derive(Serialize, Deserialize)]
/// A method used for the primary authentication for a user.
pub enum PrimaryAuthenticationMethod {
    /// Standard password authentication.
    Password {
        /// The actual internal password data.
        password: String,
    },
}

async fn do_password_authentication(
    user_id: u64,
    password: &str,
    db_conn: &PgPool,
) -> Result<bool, sqlx::Error> {
    Ok(Password::select(user_id, db_conn)
        .await?
        .is_some_and(|fetched| fetched.verify(password)))
}

impl PrimaryAuthenticationMethod {
    /// Authenticate using this authentication method.
    async fn authenticate(self, user_id: u64, db_conn: &PgPool) -> Result<bool, sqlx::Error> {
        match self {
            Self::Password { password } => {
                do_password_authentication(user_id, &password, db_conn).await
            }
        }
    }
}

#[derive(Serialize, Deserialize)]
/// A method used for MFA for a user.
pub enum MfaAuthenticationMethod {
    /// Time-Based One-Time-Password code.
    Totp {
        /// The generated TOTP code.
        code: String,
    },
}

/// List all supported authentication methods.
pub fn list_supported_authentication_methods() -> Vec<PrimaryAuthenticationMethod> {
    vec![PrimaryAuthenticationMethod::Password {
        password: "string".to_owned(),
    }]
}

/// Authenticate with a primary authentication method, and return a session
/// token if successful. The session token can be Full or Partial. If it is
/// Partial, then further authentication (2fa) is required.
pub async fn authenticate(
    email: &str,
    credential: PrimaryAuthenticationMethod,
    db_conn: &PgPool,
    redis_conn: &mut MultiplexedConnection,
) -> Result<Option<SessionToken>, sqlx::Error> {
    let res = AppUser::select_by_email(email, db_conn).await?;
    let Some(user) = res else { return Ok(None) };
    if !credential.authenticate(user.id(), db_conn).await? {
        return Ok(None);
    }
    Ok(Some(
        SessionToken::create(
            user.id(),
            match Totp::select(user.id(), db_conn).await? {
                None => SessionTokenType::Full,
                Some(_) => SessionTokenType::Partial,
            },
            redis_conn,
        )
        .await
        .expect("Whoops, Redis error"),
    ))
}

/// The type of session token.
pub enum SessionTokenType {
    /// A regular session token fully authenticated to perform user actions.
    Full,
    /// A partial session token authorised only for initiating 2fa.
    Partial,
}

/// Represents a generic session token.
pub enum SessionToken {
    /// A regular session token fully authorised to perform user actions.
    Full(String),
    /// A partial session token authorised only for initiating 2fa.
    Partial(String),
}

impl SessionToken {
    /// Generate a cryptographically secure random 24-byte session token.
    fn generate() -> String {
        let mut rng = StdRng::from_entropy();
        let mut token_buf: [u8; 24] = [0; 24];
        rng.fill(&mut token_buf);
        token_buf
            .into_iter()
            .fold(String::new(), |acc: String, x: u8| format!("{acc}{x:x}"))
    }
    /// Create a new session token for a user.
    pub async fn create(
        user_id: u64,
        token_type: SessionTokenType,
        redis_conn: &mut MultiplexedConnection,
    ) -> redis::RedisResult<Self> {
        let mut token;
        let (hash_name, timeout) = match token_type {
            SessionTokenType::Full => ("session", SESSION_TIMEOUT),
            SessionTokenType::Partial => ("partial-session", AUTH_SESSION_TIMEOUT),
        };
        loop {
            token = Self::generate();
            let _: () = redis_conn.hset_nx(hash_name, &token, user_id).await?;
            // Continue generating if the generated token was already taken.
            let set_user_id: u64 = redis_conn.hget(hash_name, &token).await?;
            if set_user_id == user_id {
                break;
            }
        }
        let _: () = redis_conn
            .hexpire(hash_name, timeout, redis::ExpireOption::NX, &token)
            .await?;
        Ok(match token_type {
            SessionTokenType::Full => Self::Full(token),
            SessionTokenType::Partial => Self::Partial(token),
        })
    }
    /// Get the user ID authenticated by this session token.
    pub async fn user_id(
        &self,
        redis_conn: &mut MultiplexedConnection,
    ) -> redis::RedisResult<Option<u64>> {
        let (redis_hash_name, raw_token): (&str, &str) = match *self {
            Self::Full(ref token) => ("session", token),
            Self::Partial(ref token) => ("partial-session", token),
        };
        redis_conn.hget(redis_hash_name, raw_token).await
    }
}

/// List 2fa methods available for a user 
pub async fn list_mfa_methods(
    user_id: u64,
    db_conn: &PgPool,
) -> Result<Vec<MfaAuthenticationMethod>, sqlx::Error> {
    let mut methods = vec![];
    let totp_enabled = totp::Totp::select(user_id, db_conn).await?.is_some();
    if totp_enabled {
        methods.push(MfaAuthenticationMethod::Totp {
            code: "string".to_owned(),
        });
    }
    Ok(methods)
}
