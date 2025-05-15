//! Models mapping to the appuser database table. Represents a user and their
//! associated information.
#![expect(clippy::pattern_type_mismatch, reason = "SQLx enum bug")]

use crate::{
    constants::db::DB_ENCRYPTION_KEY,
    db::{errors::DatabaseError, ConnectionPool},
    utils::email::EmailAddress,
};
use serde::{Deserialize, Serialize};
use sqlx::{postgres::PgArguments, query, query_as, Arguments as _, QueryBuilder};
use uuid::Uuid;

/// INSERT model for an `AppUser`. Used ONLY when creating a new user.
#[derive(Deserialize, Clone)]
pub struct AppUserInsert {
    /// The user's email address.
    pub email: EmailAddress,
    /// The user's forename.
    pub forename: String,
    /// The user's surname.
    pub surname: String,
    /// The user's address.
    pub address: String,
}

#[derive(sqlx::Type, Serialize, PartialEq, Eq, Deserialize)]
#[sqlx(type_name = "app_user_role")]
pub enum AppUserRole {
    /// A regular customer, able to purchase items.
    Customer,
    /// An administrator, able to modify items.
    Administrator,
}

#[derive(Deserialize)]
pub struct AppUserSearchParameters {
    pub email: Option<EmailAddress>,
    pub role: Option<AppUserRole>,
}

/// An `AppUser` which is stored in the database. Can only be constructed by
/// reading it from the database.
#[derive(Serialize, sqlx::FromRow)]
pub struct AppUser {
    /// The user's ID primary key.
    id: Uuid,
    /// The user's email address.
    pub email: EmailAddress,
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
            email,
            forename: forename.to_owned(),
            surname: surname.to_owned(),
            address: address.to_owned(),
        }
    }

    /// Store this INSERT model in the database and return a complete `AppUser` model.
    pub async fn store(self, db_client: &ConnectionPool) -> Result<AppUser, DatabaseError> {
        #[expect(clippy::as_conversions, reason="Used in query_as! macro for Postgres coersion")]
        Ok(query_as!(
            AppUser,
            r#"INSERT INTO appuser
            (email, forename, surname, address, role)
            VALUES ($1, pgp_sym_encrypt($2, $5), pgp_sym_encrypt($3, $5), pgp_sym_encrypt($4, $5), 'Customer')
            RETURNING id, email AS "email: _", pgp_sym_decrypt(forename, $5) AS "forename!",
            pgp_sym_decrypt(surname, $5) AS "surname!",
            pgp_sym_decrypt(address, $5) AS "address!", role AS "role!: AppUserRole""#,
            String::from(self.email),
            self.forename,
            self.surname,
            self.address,
            *DB_ENCRYPTION_KEY
        ).fetch_one(db_client).await?)
    }
}

impl AppUser {
    /// Get the `AppUser`'s ID primary key.
    pub const fn id(&self) -> Uuid {
        self.id
    }
    /// Select an `AppUser` from the database by ID.
    pub async fn select_one(
        id: Uuid,
        db_client: &ConnectionPool,
    ) -> Result<Option<Self>, DatabaseError> {
        Ok(query_as!(
            Self,
            r#"SELECT id, email AS "email: _", pgp_sym_decrypt(forename, $2) AS "forename!",
            pgp_sym_decrypt(surname, $2) AS "surname!",
            pgp_sym_decrypt(address, $2) AS "address!",
            role AS "role!: AppUserRole" FROM appuser WHERE id = $1"#,
            id,
            *DB_ENCRYPTION_KEY
        )
        .fetch_optional(db_client)
        .await?)
    }

    /// Retrieve all `AppUser` records in the database.
    pub async fn select_all(db_client: &ConnectionPool) -> Result<Vec<Self>, DatabaseError> {
        Ok(query_as!(
            Self,
            r#"SELECT id, email AS "email: _", pgp_sym_decrypt(forename, $1) AS "forename!",
            pgp_sym_decrypt(surname, $1) AS "surname!",
            pgp_sym_decrypt(address, $1) AS "address!",
            role AS "role!: AppUserRole" FROM appuser"#,
            *DB_ENCRYPTION_KEY
        )
        .fetch_all(db_client)
        .await?)
    }
    /// Update the database record to match the model's current state.
    pub async fn update(&self, db_client: &ConnectionPool) -> Result<(), DatabaseError> {
        query!(
            "UPDATE appuser SET email = $1,
            forename = pgp_sym_encrypt($2, $6),
            surname = pgp_sym_encrypt($3, $6),
            address = pgp_sym_encrypt($4, $6) WHERE id = $5",
            String::from(self.email.clone()),
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

    pub async fn search(
        params: AppUserSearchParameters,
        db_client: &ConnectionPool,
    ) -> Result<Vec<Self>, DatabaseError> {
        let mut arguments = PgArguments::default();
        arguments
            .add(&*DB_ENCRYPTION_KEY)
            .expect("Error adding arguments to sql query builder.");
        let mut query = QueryBuilder::with_arguments(
            "SELECT id, email, pgp_sym_decrypt(forename, $1) AS forename,
            pgp_sym_decrypt(surname, $1) as surname,
            pgp_sym_decrypt(address, $1) as address,
            role
            FROM appuser WHERE 1=1",
            arguments,
        );

        if let Some(email) = params.email {
            let escaped_email = String::from(email)
                .replace('\\', "\\\\")
                .replace('%', "\\%")
                .replace('_', "\\_");
            query.push(" AND email LIKE ");
            query.push_bind(format!("%{escaped_email}%"));
        }
        if let Some(role) = params.role {
            query.push(" AND role = ");
            query.push_bind(role);
        }
        Ok(query.build_query_as().fetch_all(db_client).await?)
    }
}
