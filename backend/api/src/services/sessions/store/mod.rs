//! Provides an abstracted interface to the underlying session store. Accessible only
//! within the session service, since no other part of the code should ever access
//! the session store.
use crate::constants::redis as constants;
use redis::{aio::MultiplexedConnection, AsyncCommands as _, ExpireOption::NX};

#[derive(Clone)]
/// A connection to the session store. Guaranteed to be safe to clone and share
/// between threads.
pub struct Connection(MultiplexedConnection);

#[derive(Copy, Clone)]
pub enum SessionType {
    PreAuthentication,
    Authenticated,
    Registration,
}

impl SessionType {
    fn to_parent_key_name(self) -> String {
        match self {
            Self::PreAuthentication => String::from("sessions:preauthentication"),
            Self::Authenticated => String::from("sessions:authenticated"),
            Self::Registration => String::from("sessions:registration"),
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
    pub(super) async fn create(
        &mut self,
        token: &str,
        user_id: u64,
        session_type: SessionType,
    ) -> Result<(), errors::SessionCreationError> {
        let key = format!("{}:{token}", session_type.to_parent_key_name());
        let _: () = self.0.hset_nx(&key, "user_id", user_id).await?;
        let set_user_id: u64 = self.0.hget(&key, "user_id").await?;
        if set_user_id != user_id {
            return Err(errors::SessionCreationError::Duplicate);
        }
        Ok(())
    }

    /// Delete a token and all associated data from the store.
    pub(super) async fn delete(
        &mut self,
        token: &str,
        session_type: SessionType,
    ) -> Result<(), errors::SessionStorageError> {
        let key = format!("{}:{token}", session_type.to_parent_key_name());
        Ok(self.0.hdel(key, "user_id").await?)
    }

    /// Set a token's expiry in seconds.
    pub(super) async fn set_expiry(
        &mut self,
        token: &str,
        seconds: u32,
        session_type: SessionType,
    ) -> Result<(), errors::SessionStorageError> {
        let key = format!("{}:{token}", session_type.to_parent_key_name());
        Ok(self
            .0
            .hexpire(key, i64::from(seconds), NX, "user_id")
            .await?)
    }
    /// Get stored session info associated with a given token.
    pub(super) async fn get_user_id(
        &mut self,
        token: &str,
        session_type: SessionType,
    ) -> Result<Option<u64>, errors::SessionStorageError> {
        let key = format!("{}:{token}", session_type.to_parent_key_name());
        let result: Option<u64> = self.0.hget(&key, "user_id").await?;
        Ok(result)
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
