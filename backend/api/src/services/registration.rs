use super::sessions;
use crate::{
    db::{
        self,
        models::appuser::{AppUser, AppUserInsert},
    },
    services::sessions::RegistrationSession,
};
use serde::Deserialize;

pub async fn signup_init(
    user_data: AppUserInsert,
    session_store_conn: &mut sessions::store::Connection,
    db_conn: &db::ConnectionPool,
) -> Result<RegistrationSession, super::errors::StorageError> {
    if AppUser::select_by_email(&String::from(user_data.email()), db_conn)
        .await?
        .is_some()
    {
        panic!("User already exists");
    }
    Ok(RegistrationSession::create(session_store_conn).await?)
}

#[derive(Deserialize)]
pub enum PrimaryAuthenticationMethod {
    Password { password: String },
}
pub async fn signup_add_credential(
    registration_session: RegistrationSession,
    credential: PrimaryAuthenticationMethod,
) -> Result<(), ()> {
    todo!()
}

pub async fn signup_finalise(
    registration_session: RegistrationSession,
    db_conn: &db::ConnectionPool,
) {
    todo!()
}
