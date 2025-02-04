//! Models mapping to the appuser database table. Represents a user and their
//! associated information.
use crate::{
    db::{errors::DatabaseError, ConnectionPool},
    utils::email::EmailAddress,
};
use serde::Deserialize;
use sqlx::{query, query_as};

/// INSERT model for an `AppUser`. Used ONLY when creating a new user.
#[derive(Deserialize, Clone)]
pub struct AppUserInsert {
    /// The user's email address. Private to enforce validity.
    email: String,
    /// The user's forename.
    pub forename: String,
    /// The user's surname.
    pub surname: String,
    /// The user's age. Signed to match Postgres SMALLINT, private to enforce range.
    age: i16,
}

/// An `AppUser` which is stored in the database. Can only be constructed by
/// reading it from the database.
pub struct AppUser {
    /// The user's ID primary key.
    id: i64,
    /// The user's email address. Private to enforce validity.
    email: String,
    /// The user's forename.
    pub forename: String,
    /// The user's surname.
    pub surname: String,
    /// The user's age.
    age: i16,
}

impl AppUserInsert {
    /// Construct a new `AppUser` INSERT model.
    pub fn new(
        email: EmailAddress,
        forename: &str,
        surname: &str,
        age: u8, // ensures age is > 0, reasonable, and fits in an i16
    ) -> Self {
        Self {
            email: email.into(),
            forename: forename.to_owned(),
            surname: surname.to_owned(),
            age: i16::from(age),
        }
    }

    /// Store this INSERT model in the database and return a complete `AppUser` model.
    pub async fn store(self, db_client: &ConnectionPool) -> Result<AppUser, DatabaseError> {
        Ok(query_as!(
            AppUser,
            "INSERT INTO appuser (email, forename, surname, age) VALUES ($1, $2, $3, $4) RETURNING *",
            self.email,
            self.forename,
            self.surname,
            self.age
        ).fetch_one(db_client).await?)
    }

    pub fn email(&self) -> EmailAddress {
        EmailAddress::try_from(self.email.clone())
            .expect("Solar bit flip has changed an email address")
    }

    pub fn age(&self) -> u8 {
        u8::try_from(self.age).expect("Somehow a non-u8 value got into an AppUserInsert.")
    }
}

impl AppUser {
    /// Get the `AppUser`'s ID primary key.
    pub fn id(&self) -> u32 {
        u32::try_from(self.id)
            .expect("User ID in database out of range for u32. Time to switch to u64/numeric.")
    }
    /// Get the user's email address.
    pub fn email(&self) -> EmailAddress {
        EmailAddress::try_from(self.email.clone())
            .expect("Invalid email format read from the database.")
    }
    /// Get the user's age.
    pub fn age(&self) -> u8 {
        u8::try_from(self.age).expect("Invalid age range read from the database.")
    }
    /// Select an `AppUser` from the database by ID.
    pub async fn select_one(
        id: i64,
        db_client: &ConnectionPool,
    ) -> Result<Option<Self>, DatabaseError> {
        Ok(query_as!(Self, "SELECT * FROM appuser WHERE id = $1", &id)
            .fetch_optional(db_client)
            .await?)
    }
    /// Select an `AppUser` from the database by email.
    pub async fn select_by_email(
        email: &str,
        db_client: &ConnectionPool,
    ) -> Result<Option<Self>, DatabaseError> {
        Ok(
            query_as!(Self, "SELECT * FROM appuser WHERE email = $1", email)
                .fetch_optional(db_client)
                .await?,
        )
    }
    /// Retrieve all `AppUser` records in the database.
    pub async fn select_all(db_client: &ConnectionPool) -> Result<Vec<Self>, DatabaseError> {
        Ok(query_as!(Self, "SELECT * FROM appuser")
            .fetch_all(db_client)
            .await?)
    }
    /// Update the database record to match the model's current state.
    pub async fn update(&self, db_client: &ConnectionPool) -> Result<(), DatabaseError> {
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
    /// Delete the corresponding record from the database. Also consumes the
    /// model itself for consistency.
    pub async fn delete(self, db_client: &ConnectionPool) -> Result<(), DatabaseError> {
        query!("DELETE FROM appuser WHERE id = $1", self.id)
            .execute(db_client)
            .await?;
        Ok(())
    }
}
