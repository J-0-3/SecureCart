//! Models mapping to the totp database table. Represents a Time-Based
//! One-Time-Password secret used by the user.
use crate::{
    constants::db::DB_ENCRYPTION_KEY,
    db::{errors::DatabaseError, ConnectionPool},
};
use sqlx::{query, query_as};
use uuid::Uuid;

/// INSERT model for a `Totp`. Used ONLY when adding a new secret.
pub struct TotpInsert {
    /// The ID of the user who uses this credential.
    pub user_id: Uuid,
    /// The raw TOTP secret bytes.
    pub secret: Vec<u8>,
}

/// A `Totp` secret which is stored in the database. Can only be constructed
/// by reading it from the database.
pub struct Totp {
    /// The ID of the user who uses this credential.
    user_id: Uuid,
    /// The raw TOTP secret bytes.
    secret: Vec<u8>,
}

impl TotpInsert {
    /// Store this INSERT model in the database and return a complete `Totp` model.
    pub async fn store(&self, db_client: &ConnectionPool) -> Result<Totp, DatabaseError> {
        Ok(query_as!(
            Totp,
            "INSERT INTO totp (user_id, secret) VALUES ($1, pgp_sym_encrypt_bytea($2, $3)) RETURNING *",
            self.user_id,
            self.secret,
            *DB_ENCRYPTION_KEY
        )
        .fetch_one(db_client)
        .await?)
    }

    pub fn validate(&self, code: &str) -> bool {
        let totp = totp_rs::TOTP::from_rfc6238(
            totp_rs::Rfc6238::with_defaults(self.secret.clone())
                .expect("Non-Rfc6238-compliant secret in TOTP validation"),
        )
        .expect("Invalid URL in TOTP validation");
        totp.check_current(code)
            .expect("System time error while validating Totp code")
    }
}
impl Totp {
    /// Select a Totp record from the database by the associated user ID.
    pub async fn select(
        user_id: Uuid,
        db_client: &ConnectionPool,
    ) -> Result<Option<Self>, DatabaseError> {
        Ok(query_as!(
            Self,
            r#"SELECT user_id, pgp_sym_decrypt_bytea(secret, $2) AS "secret!" FROM totp WHERE user_id = $1"#,
            user_id,
            *DB_ENCRYPTION_KEY
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
        let totp = totp_rs::TOTP::from_rfc6238(
            totp_rs::Rfc6238::with_defaults(self.secret.clone())
                .expect("Non-Rfc6238-compliant secret in TOTP validation"),
        )
        .expect("Invalid URL in TOTP validation");
        totp.check_current(code)
            .expect("System time error while validating Totp code")
    }
}
