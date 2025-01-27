//! Models mapping to the totp database table. Represents a Time-Based
//! One-Time-Password secret used by the user.
use sqlx::{query, query_as, Error, PgPool};

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
    pub async fn store(&self, db_client: &PgPool) -> Result<(), Error> {
        query!(
            "INSERT INTO totp (user_id, secret) VALUES ($1, $2)",
            self.user_id,
            self.secret
        )
        .execute(db_client)
        .await
        .map(|_| ())
    }
}
impl Totp {
    /// Select a Totp record from the database by the associated user ID.
    pub async fn select(user_id: i64, db_client: &PgPool) -> Result<Option<Self>, Error> {
        query_as!(Self, "SELECT * FROM totp WHERE user_id = $1", user_id)
            .fetch_optional(db_client)
            .await
    }
    /// Delete the model from the database. Also consumes the model for the sake
    /// of consistency.
    pub async fn delete(self, db_client: &PgPool) -> Result<(), Error> {
        query!("DELETE FROM totp WHERE user_id = $1", self.user_id)
            .execute(db_client)
            .await
            .map(|_| ())
    }
}
