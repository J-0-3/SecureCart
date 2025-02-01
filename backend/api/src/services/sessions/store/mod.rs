use crate::constants::redis as constants;
use redis::{aio::MultiplexedConnection, AsyncCommands as _, ExpireOption::NX, RedisError};

#[derive(Clone)]
pub struct Connection(MultiplexedConnection);
pub type StorageError = RedisError;

pub(super) enum SessionCreationError {
    Duplicate,
    StorageError(StorageError),
}

impl From<StorageError> for SessionCreationError {
    fn from(err: StorageError) -> Self {
        Self::StorageError(err)
    }
}

pub(super) struct SessionInfo {
    pub user_id: u64,
    pub authenticated: bool,
}
impl Connection {
    pub async fn connect() -> Result<Self, StorageError> {
        Ok(Self(
            redis::Client::open(constants::REDIS_URL.clone())?
                .get_multiplexed_async_connection()
                .await?,
        ))
    }
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
    pub(super) async fn set_authenticated(
        &mut self,
        token: &str,
        authenticated: bool,
    ) -> Result<(), StorageError> {
        let key = format!("session:{token}");
        self.0.hset(&key, "authenticated", authenticated).await
    }
    pub(super) async fn delete(&mut self, token: &str) -> Result<(), StorageError> {
        let key = format!("session:{token}");
        self.0.hdel(key, &["user_id", "authenticated"]).await
    }
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
    pub(super) async fn info(&mut self, token: &str) -> Result<Option<SessionInfo>, StorageError> {
        let key = format!("session:{token}");
        let result: Option<(u64, bool)> = self.0.hget(&key, &["user_id", "authenticated"]).await?;
        Ok(result.map(|(user_id, authenticated)| SessionInfo {
            user_id,
            authenticated,
        }))
    }
}
