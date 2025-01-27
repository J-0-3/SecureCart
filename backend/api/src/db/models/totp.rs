use sqlx::{query, query_as, Error, PgPool};

pub struct Totp {
    pub user_id: i64,
    pub secret: Vec<u8>,
}

impl Totp {
    pub async fn select(user_id: i64, db_client: &PgPool) -> Result<Option<Self>, Error> {
        query_as!(Self, "SELECT * FROM totp WHERE user_id = $1", user_id)
            .fetch_optional(db_client)
            .await
    }
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
    pub async fn delete(self, db_client: &PgPool) -> Result<(), Error> {
        query!("DELETE FROM totp WHERE user_id = $1", self.user_id)
            .execute(db_client)
            .await
            .map(|_| ())
    }
}
