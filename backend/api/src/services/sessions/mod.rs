//! Logic for session handling. Creating, managing and revoking session tokens.
use crate::{
    constants::sessions::{
        ADMIN_SESSION_TIMEOUT, PREAUTH_SESSION_TIMEOUT, REGISTRATION_SESSION_TIMEOUT,
        SESSION_TIMEOUT,
    },
    db::models::appuser::AppUserInsert,
};
pub mod store;
use core::fmt::Write as _;
use store::{Connection, SessionInfo};

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
    /// The information stored in this session.
    session_info: store::SessionInfo, // this might seem redundant due to the
                                      // wrapper classes, but it makes working
                                      // with the underlying store much easier
}

pub trait SessionTrait: Send + Sync + Clone + Sized {
    /// Get an instance of this session type given the corresponding session token.
    async fn get(
        token: &str,
        session_store_conn: &mut store::Connection,
    ) -> Result<Option<Self>, errors::SessionStorageError>;
    /// Get the session token which identifies this session.
    fn token(&self) -> String;
    /// Delete this session, immediately invalidating it.
    async fn delete(
        self,
        session_store_conn: &mut store::Connection,
    ) -> Result<(), errors::SessionStorageError>;
}

/// A session which is guaranteed to have been fully authenticated. Can be
/// constructed either infallibly using `PreAuthenticationSession::promote`,
/// or fallibly by using `CustomerSession::get` like usual.
#[derive(Clone)]
pub struct CustomerSession {
    /// The inner session used to interact with the session store.
    session: BaseSession,
}

/// A session generated prior to full authentication of a user. This should
/// be generated once a user has successfully submitted primary credentials,
/// and used to keep track of that user as they continue with MFA.
#[derive(Clone)]
pub struct PreAuthenticationSession {
    /// The inner session used to interact with the session store.
    session: BaseSession,
}

/// A session used for onboarding a new user. Created when the registration
/// process begins, and deleted once it is complete. Used to store submitted
/// user data between phases of onboarding.
#[derive(Clone)]
pub struct RegistrationSession {
    /// The inner session used to interact with the session store.
    session: BaseSession,
}

/// A session which has been fully authenticated and authorized to have
/// administrative access. Note that this is mutally exclusive with
/// having recular authenticated user access.
#[derive(Clone)]
pub struct AdministratorSession {
    /// The inner session used to interact with the session store.
    session: BaseSession,
}

/// A generic authenticated session, which may either be a customer
/// or administrator session.
#[derive(Clone)]
pub enum GenericAuthenticatedSession {
    /// A customer session.
    Customer(CustomerSession),
    /// An administrator session.
    Administrator(AdministratorSession),
}

impl SessionTrait for GenericAuthenticatedSession {
    async fn get(
        token: &str,
        session_store_conn: &mut store::Connection,
    ) -> Result<Option<Self>, errors::SessionStorageError> {
        let session_opt =
            BaseSession::get(token, store::SessionType::Authenticated, session_store_conn).await?;
        Ok(session_opt.map(|session| {
            let (_, is_admin) = session.info().as_auth().expect(
                "Requested authenticated session, got something else. Bug/Redis is corrupted.",
            );
            if is_admin {
                Self::Administrator(AdministratorSession { session })
            } else {
                Self::Customer(CustomerSession { session })
            }
        }))
    }
    fn token(&self) -> String {
        let (Self::Customer(CustomerSession { ref session })
        | Self::Administrator(AdministratorSession { ref session })) = *self;
        session.token.clone()
    }
    async fn delete(
        self,
        session_store_conn: &mut store::Connection,
    ) -> Result<(), errors::SessionStorageError> {
        session_store_conn
            .delete(&self.token(), store::SessionType::Authenticated)
            .await
    }
}

impl SessionTrait for AdministratorSession {
    async fn get(
        token: &str,
        session_store_conn: &mut store::Connection,
    ) -> Result<Option<Self>, errors::SessionStorageError> {
        Ok(BaseSession::get(token, store::SessionType::Authenticated, session_store_conn).await?.and_then(
            |session|  {
                session
                    .info()
                    .as_auth()
                    .expect("Got non-authenticated session back from get with SessionType::Authenticated. Major bug in session store.")
                    .1
                    .then_some(Self { session })
            }
        ))
    }
    async fn delete(
        self,
        session_store_conn: &mut store::Connection,
    ) -> Result<(), errors::SessionStorageError> {
        session_store_conn
            .delete(&self.token(), store::SessionType::Authenticated)
            .await
    }
    fn token(&self) -> String {
        self.session.token.clone()
    }
}

impl AdministratorSession {
    /// Get the user ID of the admin identified by this session.
    pub fn user_id(&self) -> u32 {
        self.session
            .info()
            .as_auth()
            .expect("Tried to convert a registration session to an authentication session")
            .0
    }
}

impl SessionTrait for CustomerSession {
    async fn get(
        token: &str,
        session_store_conn: &mut store::Connection,
    ) -> Result<Option<Self>, errors::SessionStorageError> {
        Ok(
            BaseSession::get(token, store::SessionType::Authenticated, session_store_conn)
                .await?
                .and_then(|sess| {
                    let (_, is_admin) = sess.info().as_auth().expect(
                        "Malformed authenticated session data returned from store. Unrecoverable.",
                    );
                    if is_admin {
                        None
                    } else {
                        Some(Self { session: sess })
                    }
                }),
        )
    }
    fn token(&self) -> String {
        self.session.token.clone()
    }
    async fn delete(
        self,
        session_store_conn: &mut store::Connection,
    ) -> Result<(), errors::SessionStorageError> {
        session_store_conn
            .delete(&self.token(), store::SessionType::Authenticated)
            .await
    }
}

impl CustomerSession {
    pub const fn new(session: BaseSession) -> Self {
        Self { session }
    }

    /// Get the ID of the user authenticated by this session.
    pub fn user_id(&self) -> u32 {
        self.session
            .info()
            .as_auth()
            .expect("Attempted to convert a registration session to an authentication session.")
            .0
    }
}

impl PreAuthenticationSession {
    /// Create a new preauthentication session given a user ID.
    pub async fn create(
        user_id: u32,
        session_store_conn: &mut store::Connection,
    ) -> Result<Self, errors::SessionStorageError> {
        let session = BaseSession::create(
            SessionInfo::PreAuthentication { user_id },
            session_store_conn,
        )
        .await?;
        session
            .set_expiry(PREAUTH_SESSION_TIMEOUT, session_store_conn)
            .await?;
        Ok(Self { session })
    }
    /// Promote this preauthentication session to a fully authenticated one.
    /// Consumes the original session, who's token will no longer be valid,
    /// and generates a completely new session.
    pub async fn promote(
        self,
        session_store_conn: &mut store::Connection,
    ) -> Result<CustomerSession, errors::SessionStorageError> {
        session_store_conn
            .delete(&self.session.token, store::SessionType::PreAuthentication)
            .await?;
        let session = BaseSession::create(
            SessionInfo::Authenticated {
                user_id: self.session.info().as_pre_auth().expect(
                    "Attempted to promote a non-preauthentication session to an authenticated session.",
                ),
                admin: false,
            },
            session_store_conn,
        )
        .await?;
        session
            .set_expiry(SESSION_TIMEOUT, session_store_conn)
            .await?;
        Ok(CustomerSession { session })
    }

    /// Promote this session to an administrative session. Should ONLY be done
    /// if you have already verified that the user has admin authorization.
    pub async fn promote_to_admin(
        self,
        session_store_conn: &mut store::Connection,
    ) -> Result<AdministratorSession, errors::SessionStorageError> {
        session_store_conn
            .delete(&self.session.token, store::SessionType::PreAuthentication)
            .await?;
        let session = BaseSession::create(
            SessionInfo::Authenticated {
                user_id: self.session.info().as_pre_auth().expect(
                    "Attempted to promote non-preauthentication registration session to an administrative session.",
                ),
                admin: true
            },
            session_store_conn,
        )
        .await?;
        session
            .set_expiry(ADMIN_SESSION_TIMEOUT, session_store_conn)
            .await?;
        Ok(AdministratorSession { session })
    }
    /// Get the user ID associated with this session.
    pub fn user_id(&self) -> u32 {
        self.session
            .info()
            .as_pre_auth()
            .expect("Attempted to convert a registration session to a preauth session.")
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

    fn token(&self) -> String {
        self.session.token.clone()
    }

    async fn delete(
        self,
        session_store_conn: &mut store::Connection,
    ) -> Result<(), errors::SessionStorageError> {
        session_store_conn
            .delete(&self.token(), store::SessionType::PreAuthentication)
            .await
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
    fn token(&self) -> String {
        self.session.token.clone()
    }
    async fn delete(
        self,
        session_store_conn: &mut store::Connection,
    ) -> Result<(), errors::SessionStorageError> {
        session_store_conn
            .delete(&self.token(), store::SessionType::Registration)
            .await
    }
}

impl RegistrationSession {
    /// Create a registration session from a set of user data.
    pub async fn create(
        user_data: AppUserInsert,
        session_store_conn: &mut store::Connection,
    ) -> Result<Self, errors::SessionStorageError> {
        let session = BaseSession::create(
            store::SessionInfo::Registration { user_data },
            session_store_conn,
        )
        .await?;
        session
            .set_expiry(REGISTRATION_SESSION_TIMEOUT, session_store_conn)
            .await?;
        Ok(Self { session })
    }
    /// Return the user data associated with this registration session.
    pub fn user_data(&self) -> AppUserInsert {
        self.session
            .info()
            .as_registration()
            .expect("Attempted to convert an authentication session to a registration session.")
    }
}

impl BaseSession {
    /// Create a new session for a given user. This session is not considered
    /// fully authenticated until ``Self::authenticate`` is called on it.
    async fn create(
        session_info: SessionInfo,
        session_store_conn: &mut Connection,
    ) -> Result<Self, errors::SessionStorageError> {
        let token = loop {
            // Loop infinitely and return a token once we successful store the session.
            let candidate = generate_session_token();
            match session_store_conn
                .create(&candidate, session_info.clone())
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
            session_info,
        })
    }

    /// Get a session given its token and session type.
    async fn get(
        token: &str,
        session_type: store::SessionType,
        session_store_conn: &mut Connection,
    ) -> Result<Option<Self>, store::errors::SessionStorageError> {
        Ok(session_store_conn
            .get_info(token, session_type)
            .await?
            .map(|session_info| Self {
                token: token.to_owned(),
                session_info,
            }))
    }

    /// Set the expiry time (in seconds) for this session.
    async fn set_expiry(
        &self,
        seconds: u32,
        session_store_conn: &mut Connection,
    ) -> Result<(), errors::SessionStorageError> {
        session_store_conn
            .set_expiry(&self.token, seconds, self.session_info.clone().into())
            .await
    }
    /// Get this session's associated information.
    pub fn info(&self) -> SessionInfo {
        self.session_info.clone()
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
