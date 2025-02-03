//! Models mapping to the category database table. Represents a category which
//! a product could be considered as within.
use crate::db::{errors::DatabaseError, ConnectionPool};
use sqlx::{query, query_as};

/// INSERT model for a `Category`. Used ONLY when adding a new category.
pub struct CategoryInsert {
    /// The name of the category.
    pub name: String,
    /// A description of the category.
    pub description: String,
}

/// A `Category` which is stored in the database. Can only be constructed by
/// reading it from the database.
pub struct Category {
    /// The `Category`'s ID primary key.
    id: i64,
    /// The name of the category.
    pub name: String,
    /// A description of the category.
    pub description: String,
}

impl CategoryInsert {
    /// Store this INSERT model in the database and return a complete `Category`
    /// model.
    pub async fn store(&self, db_client: &ConnectionPool) -> Result<Category, DatabaseError> {
        Ok(query_as!(
            Category,
            "INSERT INTO category (name, description) VALUES ($1, $2) RETURNING *",
            self.name,
            self.description
        )
        .fetch_one(db_client)
        .await?)
    }
}

impl Category {
    /// Select a `Category` from the database by its ID.
    pub async fn select_one(
        id: i64,
        db_client: &ConnectionPool,
    ) -> Result<Option<Self>, DatabaseError> {
        Ok(query_as!(Self, "SELECT * FROM category WHERE id = $1", id)
            .fetch_optional(db_client)
            .await?)
    }
    /// Retrieves all categories stored in the database.
    pub async fn select_all(db_client: &ConnectionPool) -> Result<Vec<Self>, DatabaseError> {
        Ok(query_as!(Self, "SELECT * FROM category")
            .fetch_all(db_client)
            .await?)
    }
    /// Get the category's ID primary key.
    pub const fn id(&self) -> i64 {
        self.id
    }
    /// Update the corresponding database record to match the model's state.
    pub async fn update(&self, db_client: &ConnectionPool) -> Result<(), DatabaseError> {
        Ok(query!(
            "UPDATE category SET name = $1, description = $2 WHERE id = $3",
            self.name,
            self.description,
            self.id
        )
        .execute(db_client)
        .await
        .map(|_| ())?)
    }
    /// Delete the corresponding record from the database. Also consumes the model
    /// for the sake of consistency.
    pub async fn delete(self, db_client: &ConnectionPool) -> Result<(), DatabaseError> {
        Ok(query!("DELETE FROM category WHERE id = $1", self.id)
            .execute(db_client)
            .await
            .map(|_| ())?)
    }
}
