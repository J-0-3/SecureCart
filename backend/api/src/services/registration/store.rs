use crate::{constants::redis as constants, db::models::appuser::AppUserInsert};
use redis::{aio::MultiplexedConnection, AsyncCommands as _, ExpireOption::NX};

#[derive(Clone)]
pub struct Connection(MultiplexedConnection);

impl Connection {
    pub async fn connect() -> Result<Self, errors::RegistrationTemporaryStorageError> {
        Ok(Self(
            redis::Client::open(constants::REDIS_URL.clone())?
                .get_multiplexed_async_connection()
                .await?,
        ))
    }
}

pub mod errors {
    use redis::RedisError;
    use thiserror::Error;

    #[derive(Error, Debug)]
    #[error(transparent)]
    pub struct RegistrationTemporaryStorageError(#[from] RedisError);
}
