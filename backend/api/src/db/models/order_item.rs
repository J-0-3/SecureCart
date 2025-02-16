use sqlx::query_as;

use crate::db::{errors::DatabaseError, ConnectionPool};

pub struct OrderItemInsert {
    product_id: i64,
    order_id: i64,
    count: i64,
}

pub struct OrderItem {
    product_id: i64,
    order_id: i64,
    count: i64,
}

impl OrderItemInsert {
    pub fn new(product_id: u32, order_id: u32, count: u32) -> Self {
        Self {
            product_id: i64::from(product_id),
            order_id: i64::from(order_id),
            count: i64::from(count),
        }
    }
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
    pub async fn select_all(
        order_id: u32,
        db_client: &ConnectionPool,
    ) -> Result<Vec<Self>, DatabaseError> {
        Ok(query_as!(
            Self,
            "SELECT * FROM order_item WHERE order_id = $1",
            i64::from(order_id)
        )
        .fetch_all(db_client)
        .await?)
    }
    pub fn product_id(&self) -> u32 {
        u32::try_from(self.product_id).expect("Product ID in OrderItem exceeds u32 range.")
    }
    pub fn order_id(&self) -> u32 {
        u32::try_from(self.order_id).expect("Order ID in OrderItem exceeds u32 range.")
    }
    pub fn count(&self) -> u32 {
        u32::try_from(self.count).expect("Count in OrderItem exceeds u32 range.")
    }
}
