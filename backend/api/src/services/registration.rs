//! Logic for onboarding and user registration.
use super::sessions::{self, SessionTrait as _};
use crate::db::models::appuser::AppUserSearchParameters;
use crate::{
    constants::passwords::{PASSWORD_MAX_LENGTH, PASSWORD_MIN_LENGTH},
    db::{
        self,
        models::{
            appuser::{AppUser, AppUserInsert},
            password::PasswordInsert,
        },
    },
    services::sessions::RegistrationSession,
};
use serde::Deserialize;

/// Begin a signup session, setting the initial user information.
pub async fn signup_init(
    user_data: AppUserInsert,
    session_store_conn: &mut sessions::store::Connection,
    db_conn: &db::ConnectionPool,
) -> Result<RegistrationSession, errors::SignupInitError> {
    if !AppUser::search(
        AppUserSearchParameters {
            email: Some(user_data.email.clone()),
            role: None,
        },
        db_conn,
    )
    .await
    .map_err(errors::StorageError::from)?
    .is_empty()
    {
        return Err(errors::SignupInitError::DuplicateEmail(
            user_data.email.to_string(),
        ));
    }
    if user_data.address.is_empty() {
        Err(errors::SignupInitError::EmptyAddress)
    } else if user_data.surname.is_empty() {
        Err(errors::SignupInitError::EmptySurname)
    } else if user_data.forename.is_empty() {
        Err(errors::SignupInitError::EmptyForename)
    } else {
        Ok(RegistrationSession::create(user_data, session_store_conn)
            .await
            .map_err(errors::StorageError::from)?)
    }
}

/// The primary authentication method being registered by the user. Corresponds
/// `services::auth::PrimaryAuthenticationMethod`, but contains material used to
/// generate the authentication method, rather than to authenticate it.
#[derive(Deserialize)]
pub enum PrimaryAuthenticationMethod {
    /// Simple password authentication.
    Password {
        /// The raw password material.
        password: String,
    },
}

/// Add credentials to a user during a onboarding session and save the user data
/// to the database.
pub async fn add_credential_and_commit(
    registration_session: RegistrationSession,
    credential: PrimaryAuthenticationMethod,
    db_conn: &db::ConnectionPool,
    session_store_conn: &mut sessions::store::Connection,
) -> Result<(), errors::AddCredentialError> {
    let user_data = registration_session.user_data();
    let stored_user = user_data
        .store(db_conn)
        .await
        .map_err(|err| errors::AddCredentialError::StorageError(err.into()))?;
    match credential {
        PrimaryAuthenticationMethod::Password { password } => {
            if password.len() < PASSWORD_MIN_LENGTH {
                return Err(errors::AddCredentialError::PasswordTooShort);
            }
            if password.len() > PASSWORD_MAX_LENGTH {
                return Err(errors::AddCredentialError::PasswordTooLong);
            }
            let password_model = PasswordInsert::new(stored_user.id(), &password);
            if let Err(error) = password_model.store(db_conn).await {
                stored_user
                    .delete(db_conn)
                    .await
                    .map_err(|err| errors::AddCredentialError::StorageError(err.into()))?;
                return Err(errors::AddCredentialError::StorageError(error.into()));
            }
        }
    }
    registration_session
        .delete(session_store_conn)
        .await
        .map_err(|err| errors::AddCredentialError::StorageError(err.into()))?;
    Ok(())
}

/// Erors returned by this service.
pub mod errors {
    pub use super::super::errors::StorageError;
    use thiserror::Error;

    /// Errors returned while initiating an onboarding session.
    #[derive(Error, Debug)]
    pub enum SignupInitError {
        #[error(transparent)]
        /// An error in the underlying storage
        StorageError(#[from] StorageError),
        #[error("Email is already is use")]
        /// The signup attempt uses an email which is already registered.
        DuplicateEmail(String),
        #[error("The signup address field is empty")]
        /// TODO: add documentation
        EmptyAddress,
        #[error("The signup surname field is empty")]
        /// TODO: add documentation
        EmptySurname,
        #[error("The signup forename field is empty")]
        /// TODO: add documentation
        EmptyForename,
    }

    #[derive(Error, Debug)]
    pub enum AddCredentialError {
        /// An error in the underlying storage
        #[error(transparent)]
        StorageError(#[from] StorageError),
        /// The provided password was too short
        #[error("The password was below the minimum length")]
        PasswordTooShort,
        /// The provided password was too long
        #[error("The password was above the maximum length")]
        PasswordTooLong,
    }
}
