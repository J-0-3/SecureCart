//! Logic for onboarding and user registration.
use super::sessions::{self, SessionTrait as _};
use crate::{
    db::{
        self,
        models::{
            appuser::{AppUser, AppUserInsert, AppUserRole},
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
    if AppUser::select_by_email(&String::from(user_data.email()), db_conn)
        .await
        .map_err(errors::StorageError::from)?
        .is_some()
    {
        return Err(errors::SignupInitError::DuplicateEmail);
    }
    Ok(RegistrationSession::create(user_data, session_store_conn)
        .await
        .map_err(errors::StorageError::from)?)
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
pub async fn signup_add_credential_and_commit(
    registration_session: RegistrationSession,
    credential: PrimaryAuthenticationMethod,
    db_conn: &db::ConnectionPool,
    session_store_conn: &mut sessions::store::Connection,
) -> Result<(), errors::StorageError> {
    let user_data = registration_session.user_data();
    let stored_user = user_data.store(AppUserRole::Customer, db_conn).await?;
    match credential {
        PrimaryAuthenticationMethod::Password { password } => {
            let password_model = PasswordInsert::new(stored_user.id(), &password);
            if let Err(err) = password_model.store(db_conn).await {
                // Allow idempotency in the case of a storage failure by
                // rolling back the state of the database (sort of).
                stored_user.delete(db_conn).await?;
                return Err(err.into());
            }
        }
    }
    registration_session.delete(session_store_conn).await?;
    Ok(())
}

/// Erors returned by this service.
pub mod errors {
    pub use super::super::errors::StorageError;
    use thiserror::Error;

    /// Errors returned while initiating an onboarding session.
    #[derive(Error, Debug)]
    pub enum SignupInitError {
        /// An error in the underlying storage
        #[error(transparent)]
        StorageError(#[from] StorageError),
        /// The signup attempt uses an email which is already registered.
        #[error("Email is already is use")]
        DuplicateEmail,
    }
}
