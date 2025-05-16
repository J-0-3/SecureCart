//! Models for inserting and querying product images (the `product_image` table).
use crate::db::{errors::DatabaseError, ConnectionPool};
use sqlx::{query, query_as};
use uuid::Uuid;

/// An INSERT model for a product image. Should only be constructed/used
/// when newly adding an image to a product.
pub struct ProductImageInsert {
    /// The product ID to add the image to.
    product_id: Uuid,
    /// The path (URI) at which the image is stored.
    pub path: String,
}

impl ProductImageInsert {
    /// Create a new INSERT model for a product image.
    pub fn new(product_id: Uuid, path: &str) -> Self {
        Self {
            product_id,
            path: path.to_owned(),
        }
    }
    /// Store this model as a record in the database, and return a full
    /// ``ProductImage``.
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

/// A `product_image` record in the database, links a URI to an image with
/// a given product ID.
pub struct ProductImage {
    /// The product ID the image is linked to.
    product_id: Uuid,
    /// The path within the media store where the image is stored.
    pub path: String,
}

impl ProductImage {
    /// Retrieve a specific record for a given path associated with a given product,
    /// in order to perform U/D operations on it.
    pub async fn select(
        product_id: Uuid,
        path: &str,
        db_client: &ConnectionPool,
    ) -> Result<Option<Self>, DatabaseError> {
        Ok(query_as!(
            Self,
            "SELECT * FROM product_image WHERE product_id = $1 AND path = $2",
            product_id,
            path
        )
        .fetch_optional(db_client)
        .await?)
    }

    /// Retrieve all image paths associated with a given product.
    pub async fn select_all(
        product_id: Uuid,
        db_client: &ConnectionPool,
    ) -> Result<Vec<Self>, DatabaseError> {
        Ok(query_as!(
            Self,
            "SELECT * FROM product_image WHERE product_id = $1",
            product_id
        )
        .fetch_all(db_client)
        .await?)
    }

    /// Delete the image from the associated product. DOES NOT delete the image from
    /// the media store, only the record in the database associating it with
    /// a given product.
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
