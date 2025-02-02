//! Logic for session handling. Creating, managing and revoking session tokens.
use crate::constants::sessions::{AUTH_SESSION_TIMEOUT, SESSION_TIMEOUT};
pub mod store;
use store::{Connection, SessionCreationError, SessionInfo, StorageError};

/// Generates a new 24-byte session token using a CSPRNG.
fn generate_session_token() -> String {
    let mut token_buf: [u8; 24] = [0; 24];
    getrandom::fill(&mut token_buf).expect("Error getting OS random. Critical, aborting.");
    token_buf
        .into_iter()
        .fold(String::new(), |acc: String, x: u8| format!("{acc}{x:x}"))
}

#[derive(Clone)]
/// A session, associating a session token with a given user. *NOT* guaranteed
/// to be fully authenticated. Look at `AuthenticatedSession` for that.
pub struct Session {
    /// The session token used to identify this session.
    token: String,
    /// The user ID the session is associated with.
    user_id: u64,
}

/// A session which is guaranteed to have been fully authenticated. Can be
/// constructed either infallibly by calling `Session::authenticate` on a session which
/// was _not_ previously authenticated within the session store, or fallibly by calling
/// `AuthenticatedSession::try_from_session` on a session which _was_ previously
/// authenticated within the session store.
pub struct AuthenticatedSession {
    /// The underlying session object.
    session: Session,
}

/// Errors returned when fallibly converting an unauthenticated ``Session`` object
/// into an ``AuthenticatedSession`` object.
pub enum AuthenticatedFromSessionError {
    /// The session was not previously authenticated (via a call to ``Session::authenticate``).
    NotAuthenticated,
    /// The session is invalid, and does not exist in the store.
    InvalidSession,
    /// An error occurred while reading/writing the underlying session store.
    StorageError(StorageError),
}

impl From<StorageError> for AuthenticatedFromSessionError {
    fn from(err: StorageError) -> Self {
        Self::StorageError(err)
    }
}

impl AuthenticatedSession {
    /// Get a reference to this session's token.
    pub fn token(&self) -> &str {
        &self.session.token
    }
    /// Get the user ID authenticated by this session.
    pub const fn user_id(&self) -> u64 {
        self.session.user_id
    }

    /// Constructs an `AuthenticatedSession` from an existing authenticated
    /// Session object. This *DOES NOT* authenticate the session, and it
    /// will fail if the session is not already authenticated. Use
    /// `Session::authenticate` to convert an unauthenticated session to an
    /// authenticated one.
    pub async fn try_from_session(
        session: Session,
        session_store_conn: &mut Connection,
    ) -> Result<Self, AuthenticatedFromSessionError> {
        let session_info = session_store_conn
            .info(&session.token)
            .await?
            .ok_or(AuthenticatedFromSessionError::InvalidSession)?;
        if session_info.authenticated {
            Ok(Self { session })
        } else {
            Err(AuthenticatedFromSessionError::NotAuthenticated)
        }
    }
}

impl From<AuthenticatedSession> for Session {
    fn from(authenticated_session: AuthenticatedSession) -> Self {
        authenticated_session.session
    }
}

impl Session {
    /// Create a new session for a given user. This session is not considered
    /// fully authenticated until ``Self::authenticate`` is called on it.
    pub async fn create(
        user_id: u64,
        session_store_conn: &mut Connection,
    ) -> Result<Self, StorageError> {
        let token = loop {
            // Loop infinitely and return a token once we successful store the session.
            let candidate = generate_session_token();
            match session_store_conn
                .create(
                    &candidate,
                    SessionInfo {
                        user_id,
                        authenticated: false,
                    },
                )
                .await
            {
                Ok(()) => break candidate, // return candidate from loop
                Err(err) => match err {
                    SessionCreationError::StorageError(error) => return Err(error),
                    SessionCreationError::Duplicate => {} // keep looping
                },
            }
        };
        session_store_conn
            .set_expiry(&token, AUTH_SESSION_TIMEOUT)
            .await?;
        Ok(Self { token, user_id })
    }
    /// Get a session given its identifying session token. Returns an `Option::None`
    /// if the token is not valid.
    pub async fn get(
        token: &str,
        session_store_conn: &mut Connection,
    ) -> Result<Option<Self>, StorageError> {
        Ok(session_store_conn.info(token).await?.map(|info| Self {
            token: token.to_owned(),
            user_id: info.user_id,
        }))
    }
    /// Get this session's authenticated user ID.
    pub const fn user_id(&self) -> u64 {
        self.user_id
    }
    /// Get this session's token.
    pub fn token(&self) -> &str {
        &self.token
    }
    /// Verify that this session has been fully authenticated. This should
    /// only be called once (relative) certainty has been achieved that the
    /// associated user is who they say they are. This function returns an
    /// ``AuthenticatedSession``, and future calls to ``AuthenticatedSession::try_from_session``
    /// on ``Session`` objects with the same token will succeed.
    pub async fn authenticate(
        self,
        session_store_conn: &mut Connection,
    ) -> Result<AuthenticatedSession, StorageError> {
        session_store_conn
            .set_authenticated(&self.token, true)
            .await?;
        session_store_conn
            .set_expiry(&self.token, SESSION_TIMEOUT)
            .await?;
        Ok(AuthenticatedSession { session: self })
    }
}
