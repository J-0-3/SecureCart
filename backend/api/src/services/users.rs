//! Logic for working with application users, interacts with the `AppUser` model.
use core::fmt;

use serde::Deserialize;
use uuid::Uuid;

use crate::{
    constants::passwords::{PASSWORD_MAX_LENGTH, PASSWORD_MIN_LENGTH},
    db::{
        self,
        models::{
            appuser::{AppUser, AppUserRole, AppUserSearchParameters},
            password::Password,
            totp::{Totp, TotpInsert},
        },
    },
    utils::email::EmailAddress,
};

use super::registration;

/// Set a user's 2FA token. Requires an example code generated by the authenticator
/// to assure correctness.
pub async fn set_2fa(
    user_id: Uuid,
    secret: Vec<u8>,
    code: &str,
    db_conn: &db::ConnectionPool,
) -> Result<Totp, errors::SetTotpError> {
    let totp = TotpInsert { user_id, secret };
    if !totp.validate(code) {
        return Err(errors::SetTotpError::IncorrectCode(user_id));
    }
    Ok(totp.store(db_conn).await?)
}

/// Generate a new 2FA token and associated validator.
pub fn generate_2fa() -> Result<totp_rs::TOTP, errors::GenerateTotpError> {
    let mut secret_buf: [u8; 32] = [0; 32];
    getrandom::fill(&mut secret_buf).expect("Error getting OS random while generating 2fa token.");
    let rfc6238 = totp_rs::Rfc6238::with_defaults(secret_buf.to_vec())?;
    Ok(totp_rs::TOTP::from_rfc6238(rfc6238).expect("Invalid URL in TOTP initialisation"))
}

/// Retrieve a user's information from the database.
pub async fn retrieve_user(
    user_id: Uuid,
    db_conn: &db::ConnectionPool,
) -> Result<Option<AppUser>, errors::UserRetrievalError> {
    Ok(AppUser::select_one(user_id, db_conn).await?)
}

/// Search for a user matching a given set of search parameters (email/role).
pub async fn search_users(
    params: AppUserSearchParameters,
    db_conn: &db::ConnectionPool,
) -> Result<Vec<AppUser>, errors::UserSearchError> {
    Ok(AppUser::search(params, db_conn).await?)
}

/// Delete a user from the database.
pub async fn delete_user(
    user_id: Uuid,
    db_conn: &db::ConnectionPool,
) -> Result<(), errors::UserDeletionError> {
    Ok(AppUser::select_one(user_id, db_conn)
        .await?
        .ok_or(errors::UserDeletionError::UserNonExistent(user_id))?
        .delete(db_conn)
        .await?)
}

#[derive(Deserialize)]
/// The set of fields which can be updated for a given user in a request.
pub struct AppUserUpdate {
    /// The new email address if present
    email: Option<EmailAddress>,
    /// The new forename if present
    forename: Option<String>,
    /// The new surname if present
    surname: Option<String>,
    /// The new address if present
    address: Option<String>,
}

impl fmt::Display for AppUserUpdate {
    #[expect(
        clippy::min_ident_chars,
        reason = "f is the trait defined parameter name"
    )]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(ref email) = self.email {
            write!(f, "email={email} ")?;
        }
        if self.forename.is_some() {
            write!(f, "forename=[REDACTED] ")?;
        }
        if self.surname.is_some() {
            write!(f, "surname=[REDACTED] ")?;
        }
        if self.address.is_some() {
            write!(f, "address=[REDACTED] ")?;
        }
        Ok(())
    }
}

/// Update a given user's information
pub async fn update_user(
    user_id: Uuid,
    data: AppUserUpdate,
    db_conn: &db::ConnectionPool,
) -> Result<AppUser, errors::UserUpdateError> {
    let mut user = AppUser::select_one(user_id, db_conn)
        .await?
        .ok_or(errors::UserUpdateError::UserNonExistent(user_id))?;
    if let Some(email) = data.email {
        email.clone_into(&mut user.email);
    }
    if let Some(forename) = data.forename {
        forename.clone_into(&mut user.forename);
    }
    if let Some(surname) = data.surname {
        surname.clone_into(&mut user.surname);
    }
    if let Some(address) = data.address {
        address.clone_into(&mut user.address);
    }
    user.update(db_conn).await?;
    Ok(user)
}

/// Update a user's authentication method and primary credentials
pub async fn update_credential(
    user_id: Uuid,
    credential: registration::PrimaryAuthenticationMethod,
    db_conn: &db::ConnectionPool,
) -> Result<(), errors::CredentialUpdateError> {
    match credential {
        registration::PrimaryAuthenticationMethod::Password { password } => {
            if password.len() < PASSWORD_MIN_LENGTH {
                return Err(errors::CredentialUpdateError::PasswordTooShort(user_id));
            }
            if password.len() > PASSWORD_MAX_LENGTH {
                return Err(errors::CredentialUpdateError::PasswordTooLong(user_id));
            }
            if let Some(mut existing) = Password::select(user_id, db_conn).await? {
                existing.set_password(&password);
                existing.update(db_conn).await?;
            }
            // delete any other primary credentials
        }
    }
    Ok(())
}

/// Promote a user to have the Administrator role
pub async fn promote_user(
    user_id: Uuid,
    db_conn: &db::ConnectionPool,
) -> Result<AppUser, errors::UserPromotionError> {
    let mut user = AppUser::select_one(user_id, db_conn)
        .await?
        .ok_or(errors::UserPromotionError::UserNonExistent(user_id))?;
    if user.role == AppUserRole::Administrator {
        Err(errors::UserPromotionError::AlreadyAdministrator(user_id))
    } else {
        user.role = AppUserRole::Administrator;
        user.update(db_conn).await?;
        Ok(user)
    }
}

/// User manipulation related errors
pub mod errors {
    use thiserror::Error;
    use uuid::Uuid;

    use crate::db::errors::DatabaseError;

    #[derive(Debug, Error)]
    /// An error returned while retrieving a user from the database
    pub enum UserRetrievalError {
        #[error(transparent)]
        /// An error returned up from the database
        DatabaseError(#[from] DatabaseError),
    }

    #[derive(Debug, Error)]
    /// An error returned while searching for a user in the database
    pub enum UserSearchError {
        #[error(transparent)]
        /// An error returned up from the database
        DatabaseError(#[from] DatabaseError),
    }
    #[derive(Debug, Error)]
    /// An error returned while deleting a user from the database.
    pub enum UserDeletionError {
        #[error(transparent)]
        /// An error returned up from the database
        DatabaseError(#[from] DatabaseError),
        #[error("The user being deleted does not exist")]
        /// The user being deleted does not exist, includes the attempted UUID
        UserNonExistent(Uuid),
    }
    #[derive(Debug, Error)]
    /// An error returned while updating a user in the database.
    pub enum UserUpdateError {
        #[error(transparent)]
        /// An error returned up from the database
        DatabaseError(#[from] DatabaseError),
        #[error("The user being updated does not exist")]
        /// Te user being updated does not exist, includes the attempted UUID
        UserNonExistent(Uuid),
    }
    #[derive(Debug, Error)]
    /// An error returned while updating a user's authentication credentials
    pub enum CredentialUpdateError {
        #[error(transparent)]
        /// An error returned up from the database
        DatabaseError(#[from] DatabaseError),
        #[error("New password is too short")]
        /// A newly submitted password was too short for the password policy.
        PasswordTooShort(Uuid),
        #[error("New password is too long")]
        /// A newly submitted password was too long for the password policy.
        PasswordTooLong(Uuid),
    }
    #[derive(Debug, Error)]
    /// An error returned while promoting a user to an Administrator
    pub enum UserPromotionError {
        #[error(transparent)]
        /// An error returned up from the database
        DatabaseError(#[from] DatabaseError),
        #[error("The user being promoted does not exist")]
        /// The user being promoted did not exist, includes the attempted UUID
        UserNonExistent(Uuid),
        #[error("The user is already an administrator")]
        /// The user being promoted is already an administrator
        AlreadyAdministrator(Uuid),
    }
    #[derive(Debug, Error)]
    /// An error returned while generating a new TOTP validator
    pub enum GenerateTotpError {
        #[error(transparent)]
        /// An error returned from the TOTP RFC6238 subsystem
        Rfc6238Error(#[from] totp_rs::Rfc6238Error),
    }
    #[derive(Debug, Error)]
    /// An error returned while setting the active TOTP token for a user
    pub enum SetTotpError {
        #[error(transparent)]
        /// An error returned up from the database.
        DatabaseError(#[from] DatabaseError),
        #[error("The verification TOTP code was incorrect")]
        /// The example verification code provided was incorrect
        IncorrectCode(Uuid),
    }
}
