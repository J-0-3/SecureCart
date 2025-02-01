//! Provides an abstracted interface to the underlying session store. Accessible only
//! within the session service, since no other part of the code should ever access
//! the session store.
use crate::constants::redis as constants;
use redis::{aio::MultiplexedConnection, AsyncCommands as _, ExpireOption::NX, RedisError};

#[derive(Clone)]
/// A connection to the session store. Guaranteed to be safe to clone and share
/// between threads.
pub struct Connection(MultiplexedConnection);

pub type StorageError = RedisError;

/// Errors which can be thrown when creating a new session in the store.
pub(super) enum SessionCreationError {
    /// There is already a session with the same token.
    Duplicate,
    /// There was an error while writing to/reading from the store.
    StorageError(StorageError),
}

impl From<StorageError> for SessionCreationError {
    fn from(err: StorageError) -> Self {
        Self::StorageError(err)
    }
}

/// Information stored under a given session token.
pub(super) struct SessionInfo {
    /// The user ID associated with this session.
    pub user_id: u64,
    /// Whether the session token is sufficient to authenticate the user.
    pub authenticated: bool,
}
impl Connection {
    /// Initiate a new (multiplexed) connection to the session store.
    /// This connection can be cloned and is safe share between threads.
    pub async fn connect() -> Result<Self, StorageError> {
        Ok(Self(
            redis::Client::open(constants::REDIS_URL.clone())?
                .get_multiplexed_async_connection()
                .await?,
        ))
    }
    /// Create a new token with some associated session info.
    pub(super) async fn create(
        &mut self,
        token: &str,
        info: SessionInfo,
    ) -> Result<(), SessionCreationError> {
        let key = format!("session:{token}");
        let _: () = self.0.hset_nx(&key, "user_id", info.user_id).await?;
        let set_user_id: u64 = self.0.hget(&key, "user_id").await?;
        if set_user_id != info.user_id {
            return Err(SessionCreationError::Duplicate);
        }
        let _: () = self
            .0
            .hset_nx(&key, "authenticated", info.authenticated)
            .await?;
        Ok(())
    }
    /// Set a token's `authenticated` field to true.
    pub(super) async fn set_authenticated(
        &mut self,
        token: &str,
        authenticated: bool,
    ) -> Result<(), StorageError> {
        let key = format!("session:{token}");
        self.0.hset(&key, "authenticated", authenticated).await
    }
    /// Delete a token and all associated data from the store.
    pub(super) async fn delete(&mut self, token: &str) -> Result<(), StorageError> {
        let key = format!("session:{token}");
        self.0.hdel(key, &["user_id", "authenticated"]).await
    }
    /// Set a token's expiry in seconds.
    pub(super) async fn set_expiry(
        &mut self,
        token: &str,
        seconds: u32,
    ) -> Result<(), StorageError> {
        let key = format!("session:{token}");
        self.0
            .hexpire(key, i64::from(seconds), NX, &["user_id", "authenticated"])
            .await
    }
    /// Get stored session info associated with a given token.
    pub(super) async fn info(&mut self, token: &str) -> Result<Option<SessionInfo>, StorageError> {
        let key = format!("session:{token}");
        let result: Option<(u64, bool)> = self.0.hget(&key, &["user_id", "authenticated"]).await?;
        Ok(result.map(|(user_id, authenticated)| SessionInfo {
            user_id,
            authenticated,
        }))
    }
}
