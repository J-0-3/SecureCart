//! Models mapping to the apporder database table. Represents a user's order
//! from the store.
use crate::db::{ConnectionPool, errors::DatabaseError};
use sqlx::{query_as, query};
use time::PrimitiveDateTime;


/// INSERT model for an `AppOrder`. Used ONLY when creating a new order.
pub struct AppOrderInsert {
    /// The amount in pennies charged for this order.
    pub amount_charged: i64,
    /// The time and date the order was placed.
    pub order_placed: PrimitiveDateTime,
    /// The ID of the user who placed the order.
    pub user_id: i64,
}

/// An `AppOrder` which is stored in the database. Can only be constructed
/// by reading it from the database.
pub struct AppOrder {
    /// The `AppOrder`'s ID primary key. Private to restrict construction.
    id: i64,
    /// The amount in pennies charged for this order.
    pub amount_charged: i64,
    /// The time and date the order was placed.
    pub order_placed: PrimitiveDateTime,
    /// The ID of the user who placed the order.
    pub user_id: i64,
}


impl AppOrderInsert {
    /// Store this INSERT model in the database and return a complete `AppOrder` model.
    pub async fn store(self, db_client: &ConnectionPool) -> Result<AppOrder, DatabaseError> {
        Ok(query_as!(
            AppOrder, 
            "INSERT INTO apporder (user_id, order_placed, amount_charged) VALUES ($1, $2, $3) RETURNING *", 
            &self.user_id, &self.order_placed, &self.amount_charged
        ).fetch_one(db_client).await?)
    }
}

impl AppOrder {
    /// Get the `AppOrder`'s ID primary key.
    pub const fn id(&self) -> i64 {
        self.id
    }
    /// Select an `AppOrder` from the database by ID.
    pub async fn select_one(id: i64, db_client: &ConnectionPool) -> Result<Option<Self>, DatabaseError> {
        Ok(query_as!(Self, "SELECT * FROM apporder WHERE id = $1", &id)
            .fetch_optional(db_client)
            .await?)
    }
    /// Retrieve all `AppOrder` records in the database.
    pub async fn select_all(db_client: &ConnectionPool) -> Result<Vec<Self>, DatabaseError> {
        Ok(query_as!(Self, "SELECT * FROM apporder")
            .fetch_all(db_client)
            .await?)
    }
    /// Update the database record to match the model's current state.
    pub async fn update(&self, db_client: &ConnectionPool) -> Result<(), DatabaseError> {
        query!(
            "UPDATE apporder SET user_id=$1, order_placed=$2, amount_charged=$3 WHERE id=$4", 
            self.user_id, self.order_placed, self.amount_charged, self.id
        ).execute(db_client).await?;
        Ok(())
    }
    /// Delete the corresponding record from the database. Also consumes the
    /// model itself for consistency.
    pub async fn delete(self, db_client: &ConnectionPool) -> Result<(), DatabaseError> {
        query!("DELETE FROM apporder WHERE id = $1", self.id).execute(db_client).await?;
        Ok(())
    }
}

