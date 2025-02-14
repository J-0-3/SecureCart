//! Models mapping to the appuser database table. Represents a user and their
//! associated information.
#![expect(clippy::pattern_type_mismatch, reason = "SQLx enum bug")]
use crate::{
    constants::db::DB_ENCRYPTION_KEY,
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
    /// The user's address.
    pub address: String,
}

#[derive(sqlx::Type)]
#[sqlx(type_name = "app_user_role")]
pub enum AppUserRole {
    /// A regular customer, able to purchase items.
    Customer,
    /// An administrator, able to modify items.
    Administrator,
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
    pub address: String,
    /// The user's role (customer or admin).
    pub role: AppUserRole,
}

impl AppUserInsert {
    /// Construct a new `AppUser` INSERT model.
    pub fn new(email: EmailAddress, forename: &str, surname: &str, address: &str) -> Self {
        Self {
            email: email.into(),
            forename: forename.to_owned(),
            surname: surname.to_owned(),
            address: address.to_owned(),
        }
    }

    /// Store this INSERT model in the database and return a complete `AppUser` model.
    pub async fn store(
        self,
        role: AppUserRole,
        db_client: &ConnectionPool,
    ) -> Result<AppUser, DatabaseError> {
        #[expect(clippy::as_conversions, reason="Used in query_as! macro for Postgres coersion")]
        Ok(query_as!(
            AppUser,
            r#"INSERT INTO appuser
            (email, forename, surname, address, role)
            VALUES ($1, pgp_sym_encrypt($2, $6), pgp_sym_encrypt($3, $6), pgp_sym_encrypt($4, $6), $5)
            RETURNING id, email, pgp_sym_decrypt(forename, $6) AS "forename!",
            pgp_sym_decrypt(surname, $6) AS "surname!",
            pgp_sym_decrypt(address, $6) AS "address!", role AS "role!: AppUserRole""#,
            self.email,
            self.forename,
            self.surname,
            self.address,
            role as AppUserRole,
            *DB_ENCRYPTION_KEY
        ).fetch_one(db_client).await?)
    }

    /// Return the email address to store.
    pub fn email(&self) -> EmailAddress {
        EmailAddress::try_from(self.email.clone())
            .expect("Solar bit flip has changed an email address")
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
    /// Select an `AppUser` from the database by ID.
    pub async fn select_one(
        id: u32,
        db_client: &ConnectionPool,
    ) -> Result<Option<Self>, DatabaseError> {
        Ok(query_as!(
            Self,
            r#"SELECT id, email, pgp_sym_decrypt(forename, $2) AS "forename!",
            pgp_sym_decrypt(surname, $2) AS "surname!",
            pgp_sym_decrypt(address, $2) AS "address!",
            role AS "role!: AppUserRole" FROM appuser WHERE id = $1"#,
            i64::from(id),
            *DB_ENCRYPTION_KEY
        )
        .fetch_optional(db_client)
        .await?)
    }
    /// Select an `AppUser` from the database by email.
    pub async fn select_by_email(
        email: &str,
        db_client: &ConnectionPool,
    ) -> Result<Option<Self>, DatabaseError> {
        Ok(query_as!(
            Self,
            r#"SELECT id, email, pgp_sym_decrypt(forename, $2) AS "forename!",
            pgp_sym_decrypt(surname, $2) AS "surname!",
            pgp_sym_decrypt(address, $2) AS "address!",
            role AS "role!: AppUserRole" FROM appuser WHERE email = $1"#,
            email,
            *DB_ENCRYPTION_KEY
        )
        .fetch_optional(db_client)
        .await?)
    }
    /// Retrieve all `AppUser` records in the database.
    pub async fn select_all(db_client: &ConnectionPool) -> Result<Vec<Self>, DatabaseError> {
        Ok(query_as!(
            Self,
            r#"SELECT id, email, pgp_sym_decrypt(forename, $1) AS "forename!", pgp_sym_decrypt(surname, $1) AS "surname!", pgp_sym_decrypt(address, $1) AS "address!", role AS "role!: AppUserRole" FROM appuser"#,
            *DB_ENCRYPTION_KEY
        )
        .fetch_all(db_client)
        .await?)
    }
    /// Update the database record to match the model's current state.
    pub async fn update(&self, db_client: &ConnectionPool) -> Result<(), DatabaseError> {
        query!(
            "UPDATE appuser SET email = $1, forename = pgp_sym_encrypt($2, $6), surname = pgp_sym_encrypt($3, $6), address = pgp_sym_encrypt($4, $6) WHERE id = $5",
            self.email,
            self.forename,
            self.surname,
            self.address,
            self.id,
            *DB_ENCRYPTION_KEY
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
