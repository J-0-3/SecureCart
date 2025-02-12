//! Models mapping to the product database table. Represents a purchaseable
//! product in the store.
use serde::{Deserialize, Serialize};
use sqlx::{query, query_as};
use crate::db::{ConnectionPool, errors::DatabaseError};

/// INSERT model for a `product`. Used ONLY when adding a new product.
#[derive(Deserialize)]
pub struct ProductInsert {
    /// The name of the product.
    pub name: String,
    /// A description of the product.
    pub description: String,
    /// Whether the product is in stock (should be listed).
    listed: bool, 
    /// The price of the product in pennies (GBP).
    price: i64,
}

/// A `Product` which is stored in the database. Can only be constructed by
/// reading it from the database.
#[derive(Serialize)]
pub struct Product {
    /// The product's ID primary key.
    id: i64,
    /// The name of the product.
    pub name: String,
    /// A description of the product.
    pub description: String,
    /// Whether the product is in stock (should be listed).
    listed: bool,
    /// The price of the product in pennies (GBP).
    price: i64,
}

impl ProductInsert {
    /// Construct a new product INSERT model.
    pub fn new(name: &str, description: &str, listed: bool, price: u32) -> Self {
        Self {
            name: name.to_owned(),
            description: description.to_owned(),
            listed,
            price: i64::from(price),
        }
    }
    /// Get whether the product should be listed.
    pub const fn is_listed(&self) -> bool {
        self.listed
    }
    /// Get the price of the product in pennies (GBP).
    pub fn price(&self) -> u32 {
        u32::try_from(self.price).expect("Price value is invalid within model. This should never happen.")
    }
    /// Store this INSERT model in the database and return a complete `Product` model.
    pub async fn store(self, db_client: &ConnectionPool) -> Result<Product, DatabaseError> {
        Ok(query_as!(
            Product, 
            "INSERT INTO product (name, description, listed, price) VALUES ($1, $2, $3, $4) RETURNING *",
            self.name, self.description, self.listed, self.price
        ).fetch_one(db_client).await?)
    }
}

impl Product {
    /// Select a `Product` from the database by its ID.
    pub async fn select_one(id: u32, db_client: &ConnectionPool) -> Result<Option<Self>, DatabaseError> {
        Ok(query_as!(Self, "SELECT * FROM product WHERE id = $1", i64::from(id))
            .fetch_optional(db_client)
            .await?)
    }
    /// Retrieve all `Product`s stored in the database.
    pub async fn select_all(db_client: &ConnectionPool) -> Result<Vec<Self>, DatabaseError> {
        Ok(query_as!(Self, "SELECT * FROM product").fetch_all(db_client).await?)
    }
    /// Set this product as listed.
    pub fn list(&mut self) {
        self.listed = true;
    }
    /// Set this product as not listed.
    pub fn unlist(&mut self) {
        self.listed = false;
    }
    /// Set the product's price in pennies (GBP).
    pub fn set_price(&mut self, price: u32) {
        self.price = i64::from(price);
    }
    /// Set the product's description.
    pub fn set_description(&mut self, description: &str) {
        description.clone_into(&mut self.description);
    }
    /// Get whether this product is listed.
    pub const fn is_listed(&self) -> bool {
        self.listed
    }
    /// Get the price of this product in pennies (GBP).
    pub fn price(&self) -> u32 {
        u32::try_from(self.price).expect("Price value in database is out of allowed range")
    }
    /// Get this product's ID primary key.
    pub fn id(&self) -> u32 {
        u32::try_from(self.id).expect("Product ID in database out of allowed range")
    }
    /// Update the corresponding database record to match this model's state.
    pub async fn update(&self, db_client: &ConnectionPool) -> Result<(), DatabaseError> {
        Ok(query!(
            "UPDATE product SET name = $1, description = $2, listed = $3, price = $4 WHERE id = $5", 
            self.name, self.description, self.listed, self.price, self.id
        ).execute(db_client).await.map(|_| ())?)
    }
    /// Delete the corresponding record from the database. Also consumes the
    /// model for the sake of consistency.
    pub async fn delete(self, db_client: &ConnectionPool) -> Result<(), DatabaseError> {
        Ok(query!("DELETE FROM product WHERE id = $1", self.id)
            .execute(db_client)
            .await
            .map(|_| ())?)
    }

    /// Set the product's name.
    pub fn set_name(&mut self, name: &str) {
        name.clone_into(&mut self.name);
    }
}
