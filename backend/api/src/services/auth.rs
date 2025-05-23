//! Controllers which manage authentication.
use crate::{
    db::{
        self,
        models::{
            appuser::{AppUser, AppUserRole, AppUserSearchParameters},
            password::Password,
            totp::Totp,
        },
    },
    services::sessions::{self, CustomerSession, PreAuthenticationSession},
    utils::email::EmailAddress,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::sessions::AdministratorSession;

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
    user_id: Uuid,
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
        user_id: Uuid,
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

/// The outcome of an authentication attempt.
pub enum AuthenticationOutcome {
    /// The authentication was successful, and an ``AuthenticatedSession`` was created.
    Success(CustomerSession),
    /// The authentication was succesful, but further authentication is required. A
    /// ``PreAuthenticationSession`` was created.
    Partial(PreAuthenticationSession),
    /// The authentication was unsuccessful.
    Failure,
    /// The authentication was successful, and an ``AdministrativeSession`` was created.
    SuccessAdministrative(AdministratorSession),
}
/// Authenticate with a primary authentication method, and return a session
/// if successful. The session is not guaranteed to be fully authenticated,
/// and checking that `AuthenticatedSession::try_from_session` is successful
/// is recommended. If the session is not authenticated, then further action
/// (most likely MFA) is required.
pub async fn authenticate(
    email: EmailAddress,
    credential: PrimaryAuthenticationMethod,
    db_conn: &db::ConnectionPool,
    session_store_conn: &mut sessions::store::Connection,
) -> Result<AuthenticationOutcome, super::errors::StorageError> {
    let mut res = AppUser::search(
        AppUserSearchParameters {
            email: Some(email),
            role: None,
        },
        db_conn,
    )
    .await?;
    let Some(user) = res.pop() else {
        return Ok(AuthenticationOutcome::Failure);
    };
    if !credential.authenticate(user.id(), db_conn).await? {
        return Ok(AuthenticationOutcome::Failure);
    }
    let user_id = user.id();
    let session = PreAuthenticationSession::create(user_id, session_store_conn).await?;
    if Totp::select(user_id, db_conn).await?.is_none() {
        match user.role {
            AppUserRole::Customer => Ok(AuthenticationOutcome::Success(
                session.promote(session_store_conn).await?,
            )),
            AppUserRole::Administrator => Ok(AuthenticationOutcome::SuccessAdministrative(
                session.promote_to_admin(session_store_conn).await?,
            )),
        }
    } else {
        Ok(AuthenticationOutcome::Partial(session))
    }
}

/// List 2fa methods available for a user
pub async fn list_mfa_methods(
    user_id: Uuid,
    db_conn: &db::ConnectionPool,
) -> Result<Vec<MfaAuthenticationMethod>, super::errors::StorageError> {
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
    user_id: Uuid,
    method: MfaAuthenticationMethod,
    db_conn: &db::ConnectionPool,
) -> Result<bool, super::errors::StorageError> {
    match method {
        MfaAuthenticationMethod::Totp { code } => {
            let totp_secret = Totp::select(user_id, db_conn).await?;
            Ok(totp_secret.is_some_and(|secret| secret.validate(&code)))
        }
    }
}

/// Outcome of a 2-factor authentication attempt.
pub enum AuthenticationOutcome2fa {
    /// The authentication was successful, an `AuthenticatedSession` was created.
    Success(CustomerSession),
    /// The authentication was successful, an `AdministrativeSession` was created.
    SuccessAdministrative(AdministratorSession),
    /// The authentication was unsuccessful.
    Failure,
}

/// Authenticate a partially authenticated user using an MFA method.
pub async fn authenticate_2fa(
    session: PreAuthenticationSession,
    method: MfaAuthenticationMethod,
    db_conn: &db::ConnectionPool,
    session_store_conn: &mut sessions::store::Connection,
) -> Result<AuthenticationOutcome2fa, super::errors::StorageError> {
    let user = AppUser::select_one(session.user_id(), db_conn)
        .await?
        .expect("User was deleting while authenticating session. Bailing.");
    if validate_2fa(session.user_id(), method, db_conn).await? {
        match user.role {
            AppUserRole::Customer => Ok(AuthenticationOutcome2fa::Success(
                session.promote(session_store_conn).await?,
            )),
            AppUserRole::Administrator => Ok(AuthenticationOutcome2fa::SuccessAdministrative(
                session.promote_to_admin(session_store_conn).await?,
            )),
        }
    } else {
        Ok(AuthenticationOutcome2fa::Failure)
    }
}

/// Errors returned by functions within this module.
pub mod errors {}
