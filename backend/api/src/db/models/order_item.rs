//! The database model for an item within an order. Corresponds to the `OrderItem` table.
use sqlx::query_as;
use uuid::Uuid;

use crate::db::{errors::DatabaseError, ConnectionPool};

/// TODO: add documentation
pub struct OrderItemInsert {
    /// TODO: add documentation
    product_id: Uuid,
    /// TODO: add documentation
    order_id: Uuid,
    /// TODO: add documentation
    count: i64,
}

/// TODO: add documentation
pub struct OrderItem {
    /// TODO: add documentation
    product_id: Uuid,
    /// TODO: add documentation
    order_id: Uuid,
    /// TODO: add documentation
    count: i64,
}

impl OrderItemInsert {
    /// TODO: add documentation
    pub fn new(product_id: Uuid, order_id: Uuid, count: u32) -> Self {
        Self {
            product_id,
            order_id,
            count: i64::from(count),
        }
    }
    /// TODO: add documentation
    pub async fn store(self, db_client: &ConnectionPool) -> Result<OrderItem, DatabaseError> {
        Ok(query_as!(
            OrderItem,
            "INSERT INTO order_item (product_id, order_id, count) VALUES ($1, $2, $3) RETURNING *",
            self.product_id,
            self.order_id,
            self.count
        )
        .fetch_one(db_client)
        .await?)
    }
}

impl OrderItem {
    /// TODO: add documentation
    pub async fn select_all(
        order_id: Uuid,
        db_client: &ConnectionPool,
    ) -> Result<Vec<Self>, DatabaseError> {
        Ok(query_as!(
            Self,
            "SELECT * FROM order_item WHERE order_id = $1",
            order_id
        )
        .fetch_all(db_client)
        .await?)
    }
    /// TODO: add documentation
    pub const fn product_id(&self) -> Uuid {
        self.product_id
    }
    /// TODO: add documentation
    pub const fn order_id(&self) -> Uuid {
        self.order_id
    }
    /// TODO: add documentation
    pub fn count(&self) -> u32 {
        u32::try_from(self.count).expect("Count in OrderItem exceeds u32 range.")
    }
}
