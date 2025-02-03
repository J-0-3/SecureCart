//! Controllers which manage authentication.
use core::convert;

use crate::{
    db::{
        self,
        models::{appuser::AppUser, password::Password, totp::Totp},
    },
    services::sessions::{self, AuthenticatedSession, Session},
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
) -> Result<bool, db::errors::DatabaseError> {
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
    ) -> Result<bool, db::errors::DatabaseError> {
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
/// if successful. The session is not guaranteed to be fully authenticated,
/// and checking that `AuthenticatedSession::try_from_session` is successful
/// is recommended. If the session is not authenticated, then further action
/// (most likely MFA) is required.
pub async fn authenticate(
    email: &str,
    credential: PrimaryAuthenticationMethod,
    db_conn: &db::ConnectionPool,
    session_store_conn: &mut sessions::store::Connection,
) -> Result<Option<Session>, errors::StorageError> {
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
) -> Result<Vec<MfaAuthenticationMethod>, errors::StorageError> {
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
) -> Result<bool, errors::StorageError> {
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
    session_store_conn: &mut sessions::store::Connection,
) -> Result<Option<AuthenticatedSession>, errors::StorageError> {
    if validate_2fa(session.user_id(), method, db_conn).await? {
        Ok(Some(session.authenticate(session_store_conn).await?))
    } else {
        Ok(None)
    }
}

pub mod errors {
    use crate::{db::errors::DatabaseError, services::sessions::errors::SessionStorageError};
    use thiserror::Error;

    #[derive(Error, Debug)]
    pub enum StorageError {
        #[error(transparent)]
        DatabaseError(#[from] DatabaseError),
        #[error(transparent)]
        SessionStorageError(#[from] SessionStorageError),
    }
}
