use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Algorithm, Argon2, Params, Version,
};
use sqlx::{query, query_as, PgPool};

pub struct Password {
    user_id: i64,
    password: String,
}

fn create_argon2<'a>() -> Argon2<'a> {
    Argon2::new(
        Algorithm::Argon2id,
        Version::V0x13,
        Params::new(12288, 3, 1, None).expect("Invalid Argon2id parameters"),
    )
}

fn hash_password(password: &str) -> String {
    let argon2 = create_argon2();
    let salt = SaltString::generate(&mut OsRng);
    let hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .expect("Argon2id error while hashing password");
    hash.to_string()
}

impl Password {
    pub fn new(user_id: i64, password: &str) -> Self {
        Self {
            user_id,
            password: hash_password(password),
        }
    }
    pub fn verify(&self, password: &str) -> bool {
        let hash = PasswordHash::new(&self.password).expect("Argon2id hash malformed");
        let argon2 = create_argon2();
        argon2.verify_password(password.as_bytes(), &hash).is_ok()
    }
    pub fn set_password(&mut self, password: &str) {
        self.password = hash_password(password);
    }
    pub async fn select(user_id: i64, db_client: &PgPool) -> Result<Option<Self>, sqlx::Error> {
        query_as!(Self, "SELECT * FROM password WHERE user_id = $1", user_id)
            .fetch_optional(db_client)
            .await
    }
    pub async fn store(&self, db_client: &PgPool) -> Result<(), sqlx::Error> {
        query!(
            "INSERT INTO password (user_id, password) VALUES ($1, $2)",
            self.user_id,
            self.password
        )
        .execute(db_client)
        .await
        .map(|_| ())
    }
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
