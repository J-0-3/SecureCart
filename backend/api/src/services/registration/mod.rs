mod store;
use super::sessions;
use crate::{
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

#[derive(Deserialize)]
pub enum PrimaryAuthenticationMethod {
    Password { password: String },
}
pub async fn signup_add_credential_and_commit(
    registration_session: RegistrationSession,
    credential: PrimaryAuthenticationMethod,
    db_conn: &db::ConnectionPool,
) -> Result<(), errors::StorageError> {
    let user_data = registration_session.user_data();
    let stored_user = user_data.store(db_conn).await?;
    match credential {
        PrimaryAuthenticationMethod::Password { password } => {
            let password_model = PasswordInsert::new(stored_user.id(), &password);
            if let Err(e) = password_model.store(db_conn).await {
                stored_user.delete(db_conn).await?;
                return Err(e.into());
            }
        }
    }
    Ok(())
}

pub mod errors {
    pub use super::super::errors::StorageError;
    use thiserror::Error;

    #[derive(Error, Debug)]
    pub enum SignupInitError {
        #[error(transparent)]
        StorageError(#[from] StorageError),
        #[error("Email is already is use")]
        DuplicateEmail,
    }
}
