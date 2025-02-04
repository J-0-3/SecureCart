//! Models mapping to the totp database table. Represents a Time-Based
//! One-Time-Password secret used by the user.
use crate::db::{errors::DatabaseError, ConnectionPool};
use sqlx::{query, query_as};

/// INSERT model for a `Totp`. Used ONLY when adding a new secret.
pub struct TotpInsert {
    /// The ID of the user who uses this credential.
    pub user_id: i64,
    /// The raw TOTP secret bytes.
    pub secret: Vec<u8>,
}

/// A `Totp` secret which is stored in the database. Can only be constructed
/// by reading it from the database.
pub struct Totp {
    /// The ID of the user who uses this credential.
    user_id: i64,
    /// The raw TOTP secret bytes.
    secret: Vec<u8>,
}

impl TotpInsert {
    /// Store this INSERT model in the database and return a complete `Totp` model.
    pub async fn store(&self, db_client: &ConnectionPool) -> Result<(), DatabaseError> {
        Ok(query!(
            "INSERT INTO totp (user_id, secret) VALUES ($1, $2)",
            self.user_id,
            self.secret
        )
        .execute(db_client)
        .await
        .map(|_| ())?)
    }
}
impl Totp {
    /// Select a Totp record from the database by the associated user ID.
    pub async fn select(
        user_id: u32,
        db_client: &ConnectionPool,
    ) -> Result<Option<Self>, DatabaseError> {
        Ok(query_as!(
            Self,
            "SELECT * FROM totp WHERE user_id = $1",
            i64::try_from(user_id).expect("User ID out of range for Postgres BIGINT")
        )
        .fetch_optional(db_client)
        .await?)
    }
    /// Delete the model from the database. Also consumes the model for the sake
    /// of consistency.
    pub async fn delete(self, db_client: &ConnectionPool) -> Result<(), DatabaseError> {
        Ok(query!("DELETE FROM totp WHERE user_id = $1", self.user_id)
            .execute(db_client)
            .await
            .map(|_| ())?)
    }

    /// Validate that a TOTP code is correct.
    pub fn validate(&self, code: &str) -> bool {
        let totp = totp_rs::TOTP::new(totp_rs::Algorithm::SHA1, 6, 1, 30, self.secret.clone())
            .expect("Failed to initialise Totp context");
        totp.check_current(code)
            .expect("DatabaseError while validating Totp code")
    }
}
