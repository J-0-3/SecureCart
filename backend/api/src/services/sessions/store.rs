//! Provides an abstracted interface to the underlying session store. Accessible only
//! within the session service, since no other part of the code should ever access
//! the session store.
use crate::{
    constants::{
        redis as constants,
        sessions::{AUTH_PENALTY_PERIOD, AUTH_TIMEOUT_ATTEMPTS, AUTH_TIMEOUT_PERIOD},
    },
    db::models::appuser::AppUserInsert,
};
use redis::{aio::MultiplexedConnection, AsyncCommands as _};
use uuid::Uuid;

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
    /// An authenticated session.
    Authenticated,
    /// A sesssion used for onboarding.
    Registration,
}

#[derive(Clone)]
/// Information stored with a `PreAuthentication` session token.
pub struct PreAuthenticationSessionData {
    /// The ID of the user in the process of authenticating with this token.
    pub user_id: Uuid,
}

#[derive(Clone)]
/// Information stored with an Authenticated session token.
pub struct AuthenticatedSessionData {
    /// TODO: add documentation
    pub user_id: Uuid,
    /// TODO: add documentation
    pub admin: bool,
}

/// Information stored with a Registration session token.
#[derive(Clone)]
pub struct RegistrationSessionData {
    /// TODO: add documentation
    pub user_data: AppUserInsert,
}
/// Information stored alongside a session token.
#[derive(Clone)]
pub enum SessionInfo {
    /// TODO: add documentation
    PreAuthentication {
        /// TODO: add documentation
        csrf: String,
        /// TODO: add documentation
        data: PreAuthenticationSessionData,
    },
    /// TODO: add documentation
    Authenticated {
        /// TODO: add documentation
        csrf: String,
        /// TODO: add documentation
        data: AuthenticatedSessionData,
    },
    /// TODO: add documentation
    Registration {
        /// TODO: add documentation
        csrf: String,
        /// TODO: add documentation
        data: RegistrationSessionData,
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
    /// TODO: add documentation
    pub fn csrf_token(&self) -> String {
        let (Self::PreAuthentication { ref csrf, .. }
        | Self::Registration { ref csrf, .. }
        | Self::Authenticated { ref csrf, .. }) = *self;
        csrf.to_owned()
    }
    /// Extract authentication data (user ID) from this session, and return None if it is
    /// not a preauthentication session.
    pub const fn as_pre_auth(&self) -> Option<&PreAuthenticationSessionData> {
        match *self {
            Self::PreAuthentication { ref data, .. } => Some(data),
            Self::Registration { .. } | Self::Authenticated { .. } => None,
        }
    }

    /// Extract authenticated data (user ID, is admin) from this session, and return
    /// None if it is not an authenticated session.
    pub const fn as_auth(&self) -> Option<&AuthenticatedSessionData> {
        match *self {
            Self::Authenticated { ref data, .. } => Some(data),
            Self::PreAuthentication { .. } | Self::Registration { .. } => None,
        }
    }

    /// Extract user data from this, and return None if it is not a `RegistrationSession`.
    pub const fn as_registration(&self) -> Option<&RegistrationSessionData> {
        match *self {
            Self::Registration { ref data, .. } => Some(data),
            Self::PreAuthentication { .. } | Self::Authenticated { .. } => None,
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
            redis::Client::open(constants::REDIS_URL.to_owned())?
                .get_multiplexed_async_connection()
                .await?,
        ))
    }
    /// Increments an internal counter to indicate an authentication attempt, and returns whether the user is timed out or now
    pub async fn bruteforce_timeout(
        &mut self,
        client: &str,
    ) -> Result<bool, errors::SessionStorageError> {
        let key = format!("bruteforce:{client}");
        let attempts: u32 = self.0.incr(&key, 1u32).await?;
        if attempts < AUTH_TIMEOUT_ATTEMPTS {
            let _: () = self.0.expire(&key, i64::from(AUTH_TIMEOUT_PERIOD)).await?;
            Ok(false)
        } else {
            let _: () = self.0.expire(&key, i64::from(AUTH_PENALTY_PERIOD)).await?;
            Ok(true)
        }
    }
    /// Store user data for a registration session in the store.
    async fn store_registration_data(
        &mut self,
        key: &str,
        csrf: &str,
        RegistrationSessionData { user_data }: RegistrationSessionData,
    ) -> Result<(), errors::SessionCreationError> {
        let _: () = self
            .0
            .hset_nx(key, "email", user_data.email.to_string())
            .await?;
        let set_email: String = self.0.hget(key, "email").await?;
        if set_email != String::from(user_data.email) {
            return Err(errors::SessionCreationError::Duplicate);
        }
        let _: () = self
            .0
            .hset_multiple(
                key,
                &[
                    ("forename", &user_data.forename),
                    ("surname", &user_data.surname),
                    ("address", &user_data.address),
                    ("csrf", &csrf.to_owned()),
                ],
            )
            .await?;
        Ok(())
    }
    /// Store data for a regular (authenticated/preauthentication) session
    /// in the store.
    async fn store_authenticated_data(
        &mut self,
        key: &str,
        csrf: &str,
        AuthenticatedSessionData { user_id, admin }: AuthenticatedSessionData,
    ) -> Result<(), errors::SessionCreationError> {
        let _: () = self.0.hset_nx(key, "user_id", user_id).await?;
        let set_user_id: Uuid = self.0.hget(key, "user_id").await?;
        if set_user_id == user_id {
            let _: () = self.0.hset(key, "admin", admin).await?;
            let _: () = self.0.hset(key, "csrf", csrf).await?;
            Ok(())
        } else {
            Err(errors::SessionCreationError::Duplicate)
        }
    }

    /// Read a `SessionInfo::PreAuthentication` from the store with a given hash key.
    async fn store_preauthentication_data(
        &mut self,
        key: &str,
        csrf: &str,
        PreAuthenticationSessionData { user_id }: PreAuthenticationSessionData,
    ) -> Result<(), errors::SessionCreationError> {
        let _: () = self.0.hset_nx(key, "user_id", user_id).await?;
        let set_user_id: Uuid = self.0.hget(key, "user_id").await?;
        if set_user_id == user_id {
            let _: () = self.0.hset(key, "csrf", csrf).await?;
            Ok(())
        } else {
            Err(errors::SessionCreationError::Duplicate)
        }
    }

    /// Get registration user data stored in the session store for a given
    /// session.
    async fn get_registration_session_data(
        &mut self,
        key: &str,
    ) -> Result<Option<SessionInfo>, errors::SessionStorageError> {
        let email_opt: Option<String> = self.0.hget(key, "email").await?;
        let Some(email) = email_opt else {
            return Ok(None);
        };
        let forename: String = self.0.hget(key, "forename").await?;
        let surname: String = self.0.hget(key, "surname").await?;
        let address: String = self.0.hget(key, "address").await?;
        let csrf: String = self.0.hget(key, "csrf").await?;
        Ok(Some(SessionInfo::Registration {
            data: RegistrationSessionData {
                user_data: AppUserInsert::new(
                    email
                        .try_into()
                        .expect("Solar bit flip or act of God made email address invalid."),
                    &forename,
                    &surname,
                    &address,
                ),
            },
            csrf,
        }))
    }

    /// Read a `SessionInfo::Authenticated` from the session store with a given hash key.
    async fn get_authenticated_session_info(
        &mut self,
        key: &str,
    ) -> Result<Option<SessionInfo>, errors::SessionStorageError> {
        let maybe_user_id: Option<Uuid> = self.0.hget(key, "user_id").await?;
        let maybe_admin: Option<bool> = self.0.hget(key, "admin").await?;
        let maybe_csrf_token: Option<String> = self.0.hget(key, "csrf").await?;
        Ok(maybe_user_id.and_then(|user_id| {
            maybe_admin.and_then(|admin| {
                maybe_csrf_token.map(|csrf| SessionInfo::Authenticated {
                    data: AuthenticatedSessionData { user_id, admin },
                    csrf,
                })
            })
        }))
    }

    /// Read a `SessionInfo::PreAuthentication` from the session store with a given hash key.
    async fn get_preauthenticated_session_info(
        &mut self,
        key: &str,
    ) -> Result<Option<SessionInfo>, errors::SessionStorageError> {
        let maybe_user_id: Option<Uuid> = self.0.hget(key, "user_id").await?;
        let maybe_csrf_token: Option<String> = self.0.hget(key, "csrf").await?;
        Ok(maybe_user_id.and_then(|user_id| {
            maybe_csrf_token.map(|csrf| SessionInfo::PreAuthentication {
                data: PreAuthenticationSessionData { user_id },
                csrf,
            })
        }))
    }

    /// Create a new session with a given token token in the session store.
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
            SessionInfo::Registration { ref data, .. } => {
                self.store_registration_data(&key, &session_info.csrf_token(), data.to_owned())
                    .await
            }
            SessionInfo::PreAuthentication { ref data, .. } => {
                self.store_preauthentication_data(&key, &session_info.csrf_token(), data.to_owned())
                    .await
            }
            SessionInfo::Authenticated { ref data, .. } => {
                self.store_authenticated_data(&key, &session_info.csrf_token(), data.to_owned())
                    .await
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
            SessionType::PreAuthentication => self.get_preauthenticated_session_info(&key).await?,
            SessionType::Authenticated => self.get_authenticated_session_info(&key).await?,
            SessionType::Registration => self.get_registration_session_data(&key).await?,
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
