//! Models mapping to the password database table. Represents a password-based
//! credential used by a user.
use argon2::{
    password_hash::{
        rand_core::OsRng, PasswordHash, PasswordHasher as _, PasswordVerifier as _, SaltString,
    },
    Algorithm, Argon2, Params, Version,
};
use sqlx::{query, query_as, PgPool};

/// INSERT model for a `Password`. Used ONLY when adding a new credential.
pub struct PasswordInsert {
    /// The ID of the user who uses this credential.
    user_id: i64,
    /// The hashed password string.
    password: String,
}

/// A `Password` which is stored in the database. Can only be constructed
/// by reading it from the database.
pub struct Password {
    /// The ID of the user who uses this credential.
    user_id: i64,
    /// The hashed password string.
    password: String,
}

/// Instantiate an Argon2 context with the standard parameters.
fn create_argon2<'a>() -> Argon2<'a> {
    Argon2::new(
        Algorithm::Argon2id,
        Version::V0x13,
        Params::new(12288, 3, 1, None).expect("Invalid Argon2id parameters"),
    )
}

/// Convert a raw password string into a hashed representation.
fn hash_password(password: &str) -> String {
    let argon2 = create_argon2();
    let salt = SaltString::generate(&mut OsRng);
    let hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .expect("Argon2id error while hashing password");
    hash.to_string()
}

impl PasswordInsert {
    /// Construct a new password INSERT model.
    pub fn new(user_id: i64, password: &str) -> Self {
        Self {
            user_id,
            password: hash_password(password),
        }
    }
    /// Store this INSERT model in the database and return a complete `Password` model.
    pub async fn store(&self, db_client: &PgPool) -> Result<Password, sqlx::Error> {
        query_as!(
            Password,
            "INSERT INTO password (user_id, password) VALUES ($1, $2) RETURNING *",
            self.user_id,
            self.password
        )
        .fetch_one(db_client)
        .await
    }
}
impl Password {
    /// Verify that a given plaintext password matches this credential.
    pub fn verify(&self, password: &str) -> bool {
        let hash = PasswordHash::new(&self.password).expect("Argon2id hash malformed");
        let argon2 = create_argon2();
        argon2.verify_password(password.as_bytes(), &hash).is_ok()
    }
    /// Update the password stored in this credential.
    pub fn set_password(&mut self, password: &str) {
        self.password = hash_password(password);
    }
    /// Select a password credential from the database by the corresponding user's ID.
    pub async fn select(user_id: u64, db_client: &PgPool) -> Result<Option<Self>, sqlx::Error> {
        query_as!(
            Self,
            "SELECT * FROM password WHERE user_id = $1",
            i64::try_from(user_id).expect("User ID out of range for Postgres BIGINT")
        )
        .fetch_optional(db_client)
        .await
    }
    /// Update the database record to match the model's internal state.
    pub async fn update(&self, db_client: &PgPool) -> Result<(), sqlx::Error> {
        query!(
            "UPDATE password SET password = $1 WHERE user_id = $2",
            self.password,
            self.user_id
        )
        .execute(db_client)
        .await
        .map(|_| ())
    }
}
