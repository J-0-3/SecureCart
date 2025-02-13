use crate::db::{errors::DatabaseError, ConnectionPool};
use sqlx::{query, query_as};

pub struct ProductImageInsert {
    product_id: i64,
    pub path: String,
}

impl ProductImageInsert {
    pub fn new(product_id: u32, path: &str) -> Self {
        Self {
            product_id: i64::from(product_id),
            path: path.to_owned(),
        }
    }
    pub async fn store(self, db_client: &ConnectionPool) -> Result<ProductImage, DatabaseError> {
        Ok(query_as!(
            ProductImage,
            "INSERT INTO product_image (product_id, path) VALUES ($1, $2) RETURNING *",
            self.product_id,
            self.path
        )
        .fetch_one(db_client)
        .await?)
    }
}

pub struct ProductImage {
    product_id: i64,
    pub path: String,
}

impl ProductImage {
    pub async fn select(
        product_id: u32,
        path: &str,
        db_client: &ConnectionPool,
    ) -> Result<Option<Self>, DatabaseError> {
        Ok(query_as!(
            Self,
            "SELECT * FROM product_image WHERE product_id = $1 AND path = $2",
            i64::from(product_id),
            path
        )
        .fetch_optional(db_client)
        .await?)
    }
    pub async fn select_all(
        product_id: u32,
        db_client: &ConnectionPool,
    ) -> Result<Vec<Self>, DatabaseError> {
        Ok(query_as!(
            Self,
            "SELECT * FROM product_image WHERE product_id = $1",
            i64::from(product_id)
        )
        .fetch_all(db_client)
        .await?)
    }

    pub async fn delete(self, db_client: &ConnectionPool) -> Result<(), DatabaseError> {
        Ok(query!(
            "DELETE FROM product_image WHERE product_id = $1 AND path = $2",
            self.product_id,
            self.path
        )
        .execute(db_client)
        .await
        .map(|_| ())?)
    }
}
