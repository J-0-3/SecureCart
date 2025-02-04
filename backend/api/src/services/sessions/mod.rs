//! Logic for session handling. Creating, managing and revoking session tokens.
use crate::constants::sessions::{AUTH_SESSION_TIMEOUT, SESSION_TIMEOUT};
pub mod store;
use core::fmt::Write as _;
use store::Connection;

/// Generates a new 24-byte session token using a CSPRNG.
fn generate_session_token() -> String {
    let mut token_buf: [u8; 24] = [0; 24];
    getrandom::fill(&mut token_buf).expect("Error getting OS random. Critical, aborting.");
    token_buf
        .into_iter()
        .fold(String::new(), |mut acc: String, x: u8| {
            write!(acc, "{x:x}").unwrap();
            acc
        })
}

#[derive(Clone)]
/// A session, associating a session token with a given user. *NOT* guaranteed
/// to be fully authenticated. Look at `AuthenticatedSession` for that.
pub struct BaseSession {
    /// The session token used to identify this session.
    token: String,
    /// The user ID the session is associated with.
    user_id: u64,
    session_type: store::SessionType, // this might seem redundant due to the
                                      // wrapper classes, but it makes working
                                      // with the underlying store much easier
}

pub trait SessionTrait: Send + Sync + Clone {
    async fn get(
        token: &str,
        session_store_conn: &mut store::Connection,
    ) -> Result<Option<Self>, errors::SessionStorageError>
    where
        Self: Sized;
    fn info(&self) -> BaseSession;
}

/// A session which is guaranteed to have been fully authenticated. Can be
/// constructed either infallibly by calling `Session::authenticate` on a session which
/// was _not_ previously authenticated within the session store, or fallibly by calling
/// `AuthenticatedSession::try_from_session` on a session which _was_ previously
/// authenticated within the session store.
#[derive(Clone)]
pub struct AuthenticatedSession {
    /// The underlying session object.
    session: BaseSession,
}

#[derive(Clone)]
pub struct PreAuthenticationSession {
    session: BaseSession,
}

#[derive(Clone)]
pub struct RegistrationSession {
    session: BaseSession,
}

impl SessionTrait for AuthenticatedSession {
    async fn get(
        token: &str,
        session_store_conn: &mut store::Connection,
    ) -> Result<Option<Self>, errors::SessionStorageError> {
        Ok(
            BaseSession::get(token, store::SessionType::Authenticated, session_store_conn)
                .await?
                .map(|session| Self { session }),
        )
    }

    fn info(&self) -> BaseSession {
        self.session.clone()
    }
}

impl PreAuthenticationSession {
    pub async fn create(
        user_id: u64,
        session_store_conn: &mut store::Connection,
    ) -> Result<Self, errors::SessionStorageError> {
        let session = BaseSession::create(
            user_id,
            store::SessionType::PreAuthentication,
            session_store_conn,
        )
        .await?;
        session
            .set_expiry(AUTH_SESSION_TIMEOUT, session_store_conn)
            .await?;
        Ok(Self { session })
    }
    pub async fn promote(
        self,
        session_store_conn: &mut store::Connection,
    ) -> Result<AuthenticatedSession, errors::SessionStorageError> {
        session_store_conn
            .delete(&self.session.token, store::SessionType::PreAuthentication)
            .await?;
        let session = BaseSession::create(
            self.session.user_id,
            store::SessionType::Authenticated,
            session_store_conn,
        )
        .await?;
        session
            .set_expiry(SESSION_TIMEOUT, session_store_conn)
            .await?;
        Ok(AuthenticatedSession { session })
    }
}

impl SessionTrait for PreAuthenticationSession {
    async fn get(
        token: &str,
        session_store_conn: &mut store::Connection,
    ) -> Result<Option<Self>, errors::SessionStorageError> {
        Ok(BaseSession::get(
            token,
            store::SessionType::PreAuthentication,
            session_store_conn,
        )
        .await?
        .map(|session| Self { session }))
    }

    fn info(&self) -> BaseSession {
        self.session.clone()
    }
}

impl SessionTrait for RegistrationSession {
    async fn get(
        token: &str,
        session_store_conn: &mut store::Connection,
    ) -> Result<Option<Self>, errors::SessionStorageError> {
        Ok(
            BaseSession::get(token, store::SessionType::Registration, session_store_conn)
                .await?
                .map(|session| Self { session }),
        )
    }

    fn info(&self) -> BaseSession {
        self.session.clone()
    }
}

impl RegistrationSession {
    pub async fn create(
        session_store_conn: &mut store::Connection,
    ) -> Result<Self, errors::SessionStorageError> {
        Ok(Self {
            session: BaseSession::create(0, store::SessionType::Registration, session_store_conn)
                .await?,
        })
    }
}

impl BaseSession {
    /// Create a new session for a given user. This session is not considered
    /// fully authenticated until ``Self::authenticate`` is called on it.
    async fn create(
        user_id: u64,
        session_type: store::SessionType,
        session_store_conn: &mut Connection,
    ) -> Result<Self, errors::SessionStorageError> {
        let token = loop {
            // Loop infinitely and return a token once we successful store the session.
            let candidate = generate_session_token();
            match session_store_conn
                .create(&candidate, user_id, session_type)
                .await
            {
                Ok(()) => break candidate, // return candidate from loop
                Err(err) => match err {
                    store::errors::SessionCreationError::StorageError(error) => return Err(error),
                    store::errors::SessionCreationError::Duplicate => {} // keep looping
                },
            }
        };
        Ok(Self {
            token,
            user_id,
            session_type,
        })
    }

    async fn get(
        token: &str,
        session_type: store::SessionType,
        session_store_conn: &mut Connection,
    ) -> Result<Option<Self>, store::errors::SessionStorageError> {
        Ok(session_store_conn
            .get_user_id(token, session_type)
            .await?
            .map(|user_id| Self {
                token: token.to_owned(),
                user_id,
                session_type,
            }))
    }

    async fn set_expiry(
        &self,
        seconds: u32,
        session_store_conn: &mut Connection,
    ) -> Result<(), errors::SessionStorageError> {
        session_store_conn
            .set_expiry(&self.token, seconds, self.session_type)
            .await
    }
    /// Get this session's authenticated user ID.
    pub const fn user_id(&self) -> u64 {
        self.user_id
    }
    /// Get this session's token.
    pub fn token(&self) -> String {
        self.token.clone()
    }
}

/// Errors returned by function within this module.
pub mod errors {
    pub use super::store::errors::SessionStorageError;
    use thiserror::Error;

    /// Errors returned when fallibly converting an unauthenticated ``Session`` object
    /// into an ``AuthenticatedSession`` object.
    #[derive(Error, Debug)]
    pub enum SessionPromotionError {
        /// The session was not previously authenticated (via a call to ``Session::authenticate``).
        #[error("Attempted to promote an unauthenticated Session to AuthenticatedSession.")]
        NotAuthenticated,
        /// The session is invalid, and does not exist in the store.
        #[error("Attempted to promote an invalid Session to AuthenticatedSession.")]
        InvalidSession,
        /// An error occurred while reading/writing the underlying session store.
        #[error("Storage error while promoting session.")]
        StorageError(
            #[from]
            #[source]
            SessionStorageError,
        ),
    }

    #[derive(Error, Debug)]
    pub enum SessionAuthenticationError {}
}
