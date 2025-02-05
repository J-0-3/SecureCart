//! Provides an abstracted interface to the underlying session store. Accessible only
//! within the session service, since no other part of the code should ever access
//! the session store.
use crate::{constants::redis as constants, db::models::appuser::AppUserInsert};
use redis::{aio::MultiplexedConnection, AsyncCommands as _};

#[derive(Clone)]
/// A connection to the session store. Guaranteed to be safe to clone and share
/// between threads.
pub struct Connection(MultiplexedConnection);

#[derive(Copy, Clone)]
/// The type of session represented by a `SessionInfo`. Corresponds directly to
/// `SessionInfo` variants.
pub enum SessionType {
    /// A session used for authentication which is not yet complete.
    PreAuthentication,
    /// An authenticated user session.
    Authenticated,
    /// A sesssion used for onboarding.
    Registration,
}

/// Information stored alongside a session token.
#[derive(Clone)]
pub enum SessionInfo {
    /// Information stored with a `PreAuthentication` session token.
    PreAuthentication {
        /// The ID of the user in the process of authenticating with this token.
        user_id: u32,
    },
    /// Information stored with an Authenticated session token.
    Authenticated {
        /// The ID of the user authenticated by this token.
        user_id: u32,
    },
    /// Information stored with a Registration session token.
    Registration {
        /// User data to insert once the registration session completes.
        user_data: AppUserInsert,
    },
}

impl SessionType {
    /// Convert this enum to a string representing its Redis parent key name.
    /// Session data is stored under "{`SessionType::to_parent_key_name()}:{token`}"."
    fn to_parent_key_name(self) -> String {
        match self {
            Self::PreAuthentication => String::from("sessions:preauthentication"),
            Self::Authenticated => String::from("sessions:authenticated"),
            Self::Registration => String::from("sessions:registration"),
        }
    }
}

impl SessionInfo {
    /// Extract authentication data (user ID) from this session, and return an error if it is a
    /// `RegistrationSession`.
    pub const fn as_auth(&self) -> Result<u32, ()> {
        match *self {
            Self::Registration { .. } => Err(()),
            Self::PreAuthentication { user_id } | Self::Authenticated { user_id } => Ok(user_id),
        }
    }

    /// Extract user data from this, and return an error if it is not a `RegistrationSession`.
    pub fn as_registration(&self) -> Result<AppUserInsert, ()> {
        match *self {
            Self::Registration { ref user_data } => Ok(user_data.clone()),
            Self::PreAuthentication { .. } | Self::Authenticated { .. } => Err(()),
        }
    }
}

impl From<SessionInfo> for SessionType {
    fn from(value: SessionInfo) -> Self {
        match value {
            SessionInfo::PreAuthentication { .. } => Self::PreAuthentication,
            SessionInfo::Authenticated { .. } => Self::Authenticated,
            SessionInfo::Registration { .. } => Self::Registration,
        }
    }
}

impl Connection {
    /// Initiate a new (multiplexed) connection to the session store.
    /// This connection can be cloned and is safe share between threads.
    pub async fn connect() -> Result<Self, errors::SessionStorageError> {
        Ok(Self(
            redis::Client::open(constants::REDIS_URL.clone())?
                .get_multiplexed_async_connection()
                .await?,
        ))
    }
    /// Store user data for a registration session in the store.
    async fn store_registration_data(
        &mut self,
        key: &str,
        data: AppUserInsert,
    ) -> Result<(), errors::SessionCreationError> {
        let _: () = self
            .0
            .hset_nx(key, "email", String::from(data.email()))
            .await?;
        let set_email: String = self.0.hget(key, "email").await?;
        if set_email != String::from(data.email()) {
            return Err(errors::SessionCreationError::Duplicate);
        }
        let _: () = self
            .0
            .hset_multiple(
                key,
                &[("forename", &data.forename), ("surname", &data.surname)],
            )
            .await?;
        let _: () = self.0.hset(key, "age", data.age()).await?;
        Ok(())
    }
    /// Store data for a regular (authenticated/preauthentication) session
    /// in the store.
    async fn store_session_data(
        &mut self,
        key: &str,
        user_id: u32,
    ) -> Result<(), errors::SessionCreationError> {
        let _: () = self.0.hset_nx(key, "user_id", user_id).await?;
        let set_user_id: u32 = self.0.hget(key, "user_id").await?;
        if set_user_id == user_id {
            Ok(())
        } else {
            Err(errors::SessionCreationError::Duplicate)
        }
    }
    /// Get registration user data stored in the session store for a given
    /// session.
    async fn get_registration_data(
        &mut self,
        key: &str,
    ) -> Result<Option<AppUserInsert>, errors::SessionStorageError> {
        let email_opt: Option<String> = self.0.hget(key, "email").await?;
        let Some(email) = email_opt else {
            return Ok(None);
        };
        let forename: String = self.0.hget(key, "forename").await?;
        let surname: String = self.0.hget(key, "surname").await?;
        let age = self.0.hget(key, "age").await?;
        Ok(Some(AppUserInsert::new(
            email
                .try_into()
                .expect("Solar bit flip or act of God made email address invalid."),
            &forename,
            &surname,
            age,
        )))
    }

    /// Get the user ID associated with a session (will return a `SessionStorageError`
    /// if the session doesn't have a `user_id` value, which will occur only if
    /// this session is not a `PreAuthentication` or ``Authenticated`` session.)
    async fn get_session_user_id(
        &mut self,
        key: &str,
    ) -> Result<Option<u32>, errors::SessionStorageError> {
        Ok(self.0.hget(key, "user_id").await?)
    }

    /// Create a new session with a given token in the session store.
    pub(super) async fn create(
        &mut self,
        token: &str,
        session_info: SessionInfo,
    ) -> Result<(), errors::SessionCreationError> {
        let key = format!(
            "{}:{token}",
            SessionType::from(session_info.clone()).to_parent_key_name()
        );
        if self.0.exists(&key).await? {
            return Err(errors::SessionCreationError::Duplicate);
        }
        match session_info {
            SessionInfo::Registration { user_data } => {
                self.store_registration_data(&key, user_data).await
            }
            SessionInfo::PreAuthentication { user_id } | SessionInfo::Authenticated { user_id } => {
                self.store_session_data(&key, user_id).await
            }
        }
    }

    /// Delete a token and all associated data from the store.
    pub(super) async fn delete(
        &mut self,
        token: &str,
        session_type: SessionType,
    ) -> Result<(), errors::SessionStorageError> {
        let key = format!("{}:{token}", session_type.to_parent_key_name());
        let _: () = self.0.del(key).await?;
        Ok(())
    }

    /// Set a token's expiry in seconds.
    pub(super) async fn set_expiry(
        &mut self,
        token: &str,
        seconds: u32,
        session_type: SessionType,
    ) -> Result<(), errors::SessionStorageError> {
        let key = format!("{}:{token}", session_type.to_parent_key_name());
        Ok(self.0.expire(key, i64::from(seconds)).await?)
    }
    /// Get stored session info associated with a given token.
    pub(super) async fn get_info(
        &mut self,
        token: &str,
        session_type: SessionType,
    ) -> Result<Option<SessionInfo>, errors::SessionStorageError> {
        let key = format!("{}:{token}", session_type.to_parent_key_name());
        Ok(match session_type {
            SessionType::PreAuthentication => self
                .get_session_user_id(&key)
                .await?
                .map(|user_id| SessionInfo::PreAuthentication { user_id }),
            SessionType::Authenticated => self
                .get_session_user_id(&key)
                .await?
                .map(|user_id| SessionInfo::Authenticated { user_id }),
            SessionType::Registration => self
                .get_registration_data(&key)
                .await?
                .map(|user_data| SessionInfo::Registration { user_data }),
        })
    }
}

/// Errors returned by functions in this module.
pub mod errors {
    use redis::RedisError;
    use thiserror::Error;

    /// An error returned by the underlying storage layer.
    #[derive(Error, Debug)]
    #[error(transparent)]
    pub struct SessionStorageError(#[from] RedisError);

    /// Errors which can be thrown when creating a new session in the store.
    #[derive(Error, Debug)]
    pub enum SessionCreationError {
        /// There is already a session with the same token.
        #[error("Attempted to store a session token which already exists.")]
        Duplicate,
        /// There was an error while writing to/reading from the store.
        #[error(transparent)]
        StorageError(#[from] SessionStorageError),
    }

    impl From<RedisError> for SessionCreationError {
        fn from(err: RedisError) -> Self {
            Self::from(SessionStorageError::from(err))
        }
    }
}
