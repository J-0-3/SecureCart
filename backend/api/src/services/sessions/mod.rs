use crate::constants::sessions::{AUTH_SESSION_TIMEOUT, SESSION_TIMEOUT};
use rand::{rngs::StdRng, Rng as _, SeedableRng as _};
pub mod store;
use store::{Connection, SessionCreationError, SessionInfo, StorageError};

fn generate_session_token() -> String {
    let mut rng = StdRng::from_entropy();
    let mut token_buf: [u8; 24] = [0; 24];
    rng.fill(&mut token_buf);
    token_buf
        .into_iter()
        .fold(String::new(), |acc: String, x: u8| format!("{acc}{x:x}"))
}

#[derive(Clone)]
pub struct Session {
    token: String,
    user_id: u64,
}

pub struct AuthenticatedSession {
    session: Session,
}

pub enum AuthenticatedFromSessionError {
    NotAuthenticated,
    InvalidSession,
    StorageError(StorageError),
}

impl From<StorageError> for AuthenticatedFromSessionError {
    fn from(err: StorageError) -> Self {
        Self::StorageError(err)
    }
}

impl AuthenticatedSession {
    pub fn token(&self) -> &str {
        &self.session.token
    }
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
