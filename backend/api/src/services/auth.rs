//! Controllers which manage authentication.
use core::convert;

use crate::{
    db::{
        self,
        models::{appuser::AppUser, password::Password, totp::Totp},
    },
    services::sessions::{store as session_store, AuthenticatedSession, Session},
};
use serde::{Deserialize, Serialize};

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
    db_conn: &db::ConnectionPool,
) -> Result<bool, sqlx::Error> {
    Ok(Password::select(user_id, db_conn)
        .await?
        .is_some_and(|fetched| fetched.verify(password)))
}

impl PrimaryAuthenticationMethod {
    /// Authenticate using this authentication method.
    async fn authenticate(
        self,
        user_id: u64,
        db_conn: &db::ConnectionPool,
    ) -> Result<bool, sqlx::Error> {
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

#[derive(Debug)]
/// Errors related to the underlying storage layers (db, session store, etc).
pub enum StorageError {
    /// An error occurred while reading/writing the primary database.
    Database(db::StorageError),
    /// An error occurred while reading/writing the session store.
    SessionStore(session_store::StorageError),
}

impl From<db::StorageError> for StorageError {
    fn from(err: db::StorageError) -> Self {
        Self::Database(err)
    }
}
impl From<session_store::StorageError> for StorageError {
    fn from(err: session_store::StorageError) -> Self {
        Self::SessionStore(err)
    }
}
/// Authenticate with a primary authentication method, and return a session
/// if successful. The session is not guaranteed to be fully authenticated,
/// and checking that `AuthenticatedSession::try_from_session` is successful
/// is recommended. If the session is not authenticated, then further action
/// (most likely MFA) is required.
pub async fn authenticate(
    email: &str,
    credential: PrimaryAuthenticationMethod,
    db_conn: &db::ConnectionPool,
    session_store_conn: &mut session_store::Connection,
) -> Result<Option<Session>, StorageError> {
    let res = AppUser::select_by_email(email, db_conn).await?;
    let Some(user) = res else { return Ok(None) };
    if !credential.authenticate(user.id(), db_conn).await? {
        return Ok(None);
    }
    let user_id = user.id();
    let session = Session::create(user_id, session_store_conn).await?;
    if Totp::select(user_id, db_conn).await?.is_none() {
        session
            .authenticate(session_store_conn)
            .await
            .map(|sess| Some(sess.into()))
            .map_err(convert::Into::into) // why is this not implicit?
    } else {
        Ok(Some(session))
    }
}

/// List 2fa methods available for a user
pub async fn list_mfa_methods(
    user_id: u64,
    db_conn: &db::ConnectionPool,
) -> Result<Vec<MfaAuthenticationMethod>, sqlx::Error> {
    let mut methods = vec![];
    let totp_enabled = Totp::select(user_id, db_conn).await?.is_some();
    if totp_enabled {
        methods.push(MfaAuthenticationMethod::Totp {
            code: "string".to_owned(),
        });
    }
    Ok(methods)
}

/// Validate a 2fa credential for a user.
async fn validate_2fa(
    user_id: u64,
    method: MfaAuthenticationMethod,
    db_conn: &db::ConnectionPool,
) -> Result<bool, StorageError> {
    match method {
        MfaAuthenticationMethod::Totp { code } => {
            let totp_secret = Totp::select(user_id, db_conn).await?;
            Ok(totp_secret.is_some_and(|secret| secret.validate(&code)))
        }
    }
}

/// Authenticate a partially authenticated user using an MFA method.
pub async fn authenticate_2fa(
    session: Session,
    method: MfaAuthenticationMethod,
    db_conn: &db::ConnectionPool,
    session_store_conn: &mut session_store::Connection,
) -> Result<Option<AuthenticatedSession>, StorageError> {
    if validate_2fa(session.user_id(), method, db_conn).await? {
        Ok(Some(session.authenticate(session_store_conn).await?))
    } else {
        Ok(None)
    }
}
