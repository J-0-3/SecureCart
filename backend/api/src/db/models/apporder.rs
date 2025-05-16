#![expect(
    clippy::pattern_type_mismatch,
    reason = "This warning comes from sqlx::Type, not my fault"
)]
//! Models mapping to the apporder database table. Represents a user's order
//! from the store.
use crate::db::{errors::DatabaseError, ConnectionPool};
use serde::{Deserialize, Serialize, Serializer};
use sqlx::{prelude::FromRow, query, query_as, QueryBuilder};
use time::{serde::iso8601, PrimitiveDateTime};
use uuid::Uuid;

/// INSERT model for an `AppOrder`. Used ONLY when creating a new order.
pub struct AppOrderInsert {
    /// The amount in pennies charged for this order.
    pub amount_charged: i64,
    /// The time and date the order was placed.
    pub order_placed: PrimitiveDateTime,
    /// The ID of the user who placed the order.
    pub user_id: Uuid,
}

#[derive(Clone, Copy, sqlx::Type, Serialize, Deserialize, PartialEq, Eq)]
#[sqlx(type_name = "app_order_status")]
/// TODO: add documentation
pub enum AppOrderStatus {
    /// TODO: add documentation
    Unconfirmed,
    /// TODO: add documentation
    Confirmed,
    /// TODO: add documentation
    Fulfilled,
}

/// An `AppOrder` which is stored in the database. Can only be constructed
/// by reading it from the database.
#[derive(Serialize, FromRow)]
pub struct AppOrder {
    /// The `AppOrder`'s ID primary key. Private to restrict construction.
    id: Uuid,
    /// The amount in pennies charged for this order.
    pub amount_charged: i64,
    /// The time and date the order was placed.
    #[serde(serialize_with = "serialize_primitive_datetime")]
    pub order_placed: PrimitiveDateTime,
    /// The ID of the user who placed the order.
    user_id: Uuid,
    /// The order's current status.
    status: AppOrderStatus,
}

fn serialize_primitive_datetime<S>(
    time: &PrimitiveDateTime,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let utc_time = time.assume_utc();
    iso8601::serialize(&utc_time, serializer)
}

impl AppOrderInsert {
    /// Store this INSERT model in the database and return a complete `AppOrder` model.
    pub async fn store(self, db_client: &ConnectionPool) -> Result<AppOrder, DatabaseError> {
        #[expect(clippy::as_conversions, reason="As here is part of the query_as! macro")]
        Ok(query_as!(
            AppOrder,
            r#"INSERT INTO apporder (user_id, order_placed, amount_charged, status) VALUES ($1, $2, $3, $4) RETURNING id, user_id, order_placed AS "order_placed", amount_charged, status AS "status!: AppOrderStatus""#,
            &self.user_id, &self.order_placed, &self.amount_charged, AppOrderStatus::Unconfirmed as AppOrderStatus
        ).fetch_one(db_client).await?)
    }
}

#[derive(Deserialize)]
/// TODO: add documentation
pub struct AppOrderSearchParameters {
    /// TODO: add documentation
    pub user_id: Option<Uuid>,
    /// TODO: add documentation
    pub status: Option<AppOrderStatus>,
}

impl AppOrder {
    /// Get the `AppOrder`'s ID primary key.
    pub const fn id(&self) -> Uuid {
        self.id
    }
    /// TODO: add documentation
    pub const fn user_id(&self) -> Uuid {
        self.user_id
    }
    /// Select an `AppOrder` from the database by ID.
    pub async fn select_one(
        id: Uuid,
        db_client: &ConnectionPool,
    ) -> Result<Option<Self>, DatabaseError> {
        Ok(query_as!(Self, r#"SELECT id, user_id, order_placed, amount_charged, status AS "status!: AppOrderStatus" FROM apporder WHERE id = $1"#, id)
            .fetch_optional(db_client)
            .await?)
    }
    /// Retrieve all `AppOrder` records in the database.
    pub async fn select_all(db_client: &ConnectionPool) -> Result<Vec<Self>, DatabaseError> {
        Ok(query_as!(Self, r#"SELECT id, user_id, order_placed, amount_charged, status AS "status!: AppOrderStatus" FROM apporder"#)
            .fetch_all(db_client)
            .await?)
    }
    /// TODO: add documentation
    pub async fn search(
        params: AppOrderSearchParameters,
        db_client: &ConnectionPool,
    ) -> Result<Vec<Self>, DatabaseError> {
        let mut query = QueryBuilder::new(
            "SELECT id, user_id, order_placed, amount_charged, status FROM apporder WHERE 1=1",
        );
        if let Some(user_id) = params.user_id {
            query.push(" AND user_id = ");
            query.push_bind(user_id);
        }
        if let Some(status) = params.status {
            query.push(" AND status = ");
            query.push_bind(status);
        }
        Ok(query.build_query_as().fetch_all(db_client).await?)
    }

    /// Update the database record to match the model's current state.
    pub async fn update(&self, db_client: &ConnectionPool) -> Result<(), DatabaseError> {
        #[expect(clippy::as_conversions, reason="As here is part of the query! macro, not an actual as cast")]
        query!(
            "UPDATE apporder SET user_id=$1, order_placed=$2, amount_charged=$3, status=$4 WHERE id=$5",
            self.user_id, self.order_placed, self.amount_charged, self.status as AppOrderStatus, self.id
        ).execute(db_client).await?;
        Ok(())
    }
    /// Delete the corresponding record from the database. Also consumes the
    /// model itself for consistency.
    pub async fn delete(self, db_client: &ConnectionPool) -> Result<(), DatabaseError> {
        query!("DELETE FROM apporder WHERE id = $1", self.id)
            .execute(db_client)
            .await?;
        Ok(())
    }

    /// TODO: add documentation
    pub const fn status(&self) -> AppOrderStatus {
        self.status
    }
    /// TODO: add documentation
    pub const fn set_status(&mut self, status: AppOrderStatus) {
        self.status = status;
    }
}
