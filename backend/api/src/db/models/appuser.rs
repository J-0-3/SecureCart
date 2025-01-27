use crate::utils::email::EmailAddress;
use sqlx::{query, query_as, Error, PgPool};

struct AppUserInsert {
    email: String,
    pub forename: String,
    pub surname: String,
    age: i16,
}

pub struct AppUser {
    id: i64,
    email: String,
    pub forename: String,
    pub surname: String,
    age: i16,
}

impl AppUserInsert {
    pub fn new(
        email: EmailAddress,
        forename: &str,
        surname: &str,
        age: u8, // ensures age is > 0, reasonable, and fits in an i16
    ) -> Self {
        Self {
            email: email.into(),
            forename: forename.to_string(),
            surname: surname.to_string(),
            age: i16::from(age),
        }
    }

    pub async fn store(self, db_client: &PgPool) -> Result<AppUser, Error> {
        query_as!(
            AppUser,
            "INSERT INTO appuser (email, forename, surname, age) VALUES ($1, $2, $3, $4) RETURNING *",
            self.email,
            self.forename,
            self.surname,
            self.age
        ).fetch_one(db_client).await
    }
}

impl AppUser {
    pub const fn id(&self) -> i64 {
        self.id
    }
    pub fn email(&self) -> EmailAddress {
        EmailAddress::try_from(self.email.clone()).unwrap()
    }
    pub fn age(&self) -> u8 {
        u8::try_from(self.age).expect("What hte fuck")
    }
    pub async fn select_one(id: i64, db_client: &PgPool) -> Result<Option<Self>, Error> {
        query_as!(Self, "SELECT * FROM appuser WHERE id = $1", &id)
            .fetch_optional(db_client)
            .await
    }
    pub async fn select_all(db_client: &PgPool) -> Result<Vec<Self>, Error> {
        query_as!(Self, "SELECT * FROM appuser")
            .fetch_all(db_client)
            .await
    }
    pub async fn update(&self, db_client: &PgPool) -> Result<(), Error> {
        query!(
            "UPDATE appuser SET email = $1, forename = $2, surname = $3, age = $4 WHERE id = $5",
            self.email,
            self.forename,
            self.surname,
            self.age,
            self.id
        )
        .execute(db_client)
        .await?;
        Ok(())
    }

    pub async fn delete(self, db_client: &PgPool) -> Result<(), Error> {
        query!("DELETE FROM appuser WHERE id = $1", self.id)
            .execute(db_client)
            .await?;
        Ok(())
    }
}
