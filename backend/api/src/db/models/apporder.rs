//! Models mapping to the apporder database table. Represents a user's order
//! from the store.
use crate::db::{ConnectionPool, errors::DatabaseError};
use serde::Serialize;
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

#[derive(Clone, Copy, sqlx::Type, Serialize)]
#[sqlx(type_name="app_order_status")]
pub enum AppOrderStatus {
    Unconfirmed,
    Confirmed,
    Fulfilled
}

/// An `AppOrder` which is stored in the database. Can only be constructed
/// by reading it from the database.
#[derive(Serialize)]
pub struct AppOrder {
    /// The `AppOrder`'s ID primary key. Private to restrict construction.
    id: i64,
    /// The amount in pennies charged for this order.
    pub amount_charged: i64,
    /// The time and date the order was placed.
    pub order_placed: PrimitiveDateTime,
    /// The ID of the user who placed the order.
    user_id: i64,
    status: AppOrderStatus
}


impl AppOrderInsert {
    /// Store this INSERT model in the database and return a complete `AppOrder` model.
    pub async fn store(self, db_client: &ConnectionPool) -> Result<AppOrder, DatabaseError> {
        Ok(query_as!(
            AppOrder, 
            r#"INSERT INTO apporder (user_id, order_placed, amount_charged, status) VALUES ($1, $2, $3, $4) RETURNING id, user_id, order_placed, amount_charged, status AS "status!: AppOrderStatus""#, 
            &self.user_id, &self.order_placed, &self.amount_charged, AppOrderStatus::Unconfirmed as AppOrderStatus
        ).fetch_one(db_client).await?)
    }
}

impl AppOrder {
    /// Get the `AppOrder`'s ID primary key.
    pub fn id(&self) -> u32 {
        u32::try_from(self.id).expect("Order ID out of u32 range, time to upgrade integer types.")
    }
    pub fn user_id(&self) -> u32 {
        u32::try_from(self.id).expect("AppOrder User ID out of u32 range, time to upgrade integer types.")
    }
    /// Select an `AppOrder` from the database by ID.
    pub async fn select_one(id: u32, db_client: &ConnectionPool) -> Result<Option<Self>, DatabaseError> {
        Ok(query_as!(Self, r#"SELECT id, user_id, order_placed, amount_charged, status AS "status!: AppOrderStatus" FROM apporder WHERE id = $1"#, i64::from(id))
            .fetch_optional(db_client)
            .await?)
    }
    /// Retrieve all `AppOrder` records in the database.
    pub async fn select_all(db_client: &ConnectionPool) -> Result<Vec<Self>, DatabaseError> {
        Ok(query_as!(Self, r#"SELECT id, user_id, order_placed, amount_charged, status AS "status!: AppOrderStatus" FROM apporder"#)
            .fetch_all(db_client)
            .await?)
    }
    /// Retrieve all `AppOrder` records linked to a given user.
    pub async fn select_all_user(user_id: u32, db_client: &ConnectionPool) -> Result<Vec<Self>, DatabaseError> {
        Ok(query_as!(Self, r#"SELECT id, user_id, order_placed, amount_charged, status AS "status!: AppOrderStatus" FROM apporder WHERE user_id = $1"#, i64::from(user_id))
            .fetch_all(db_client)
            .await?)
    }
    /// Update the database record to match the model's current state.
    pub async fn update(&self, db_client: &ConnectionPool) -> Result<(), DatabaseError> {
        query!(
            "UPDATE apporder SET user_id=$1, order_placed=$2, amount_charged=$3, status=$4 WHERE id=$5", 
            self.user_id, self.order_placed, self.amount_charged, self.status as AppOrderStatus, self.id
        ).execute(db_client).await?;
        Ok(())
    }
    /// Delete the corresponding record from the database. Also consumes the
    /// model itself for consistency.
    pub async fn delete(self, db_client: &ConnectionPool) -> Result<(), DatabaseError> {
        query!("DELETE FROM apporder WHERE id = $1", self.id).execute(db_client).await?;
        Ok(())
    }
    pub const fn set_status(&mut self, status: AppOrderStatus) {
        self.status = status
    }
}

