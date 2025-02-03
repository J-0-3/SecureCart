//! Models mapping to the product database table. Represents a purchaseable
//! product in the store.
use sqlx::{query, query_as};
use crate::db::{ConnectionPool, errors::DatabaseError};

/// INSERT model for a `product`. Used ONLY when adding a new product.
pub struct ProductInsert {
    /// The name of the product.
    pub name: String,
    /// A description of the product.
    pub description: String,
    /// The count of the product left in stock.
    stock: i64, // i64s are used internally to match Postgres BIGINTEGER types
    /// The price of the product in pennies (GBP).
    price: i64,
}

/// A `Product` which is stored in the database. Can only be constructed by
/// reading it from the database.
pub struct Product {
    /// The product's ID primary key.
    id: i64,
    /// The name of the product.
    pub name: String,
    /// A description of the product.
    pub description: String,
    /// The count of the product left in stock.
    stock: i64,
    /// The price of the product in pennies (GBP).
    price: i64,
}

impl ProductInsert {
    /// Construct a new product INSERT model.
    pub fn new(name: &str, description: &str, stock: u32, price: u32) -> Self {
        Self {
            name: name.to_owned(),
            description: description.to_owned(),
            stock: i64::from(stock),
            price: i64::from(price),
        }
    }
    /// Get the count of the product left in stock.
    pub fn stock(&self) -> u32 {
        u32::try_from(self.stock).expect("Stock value is invalid within model. This should never happen.")
    }
    /// Get the price of the product in pennies (GBP).
    pub fn price(&self) -> u32 {
        u32::try_from(self.price).expect("Price value is invalid within model. This should never happen.")
    }
    /// Store this INSERT model in the database and return a complete `Product` model.
    pub async fn store(self, db_client: &ConnectionPool) -> Result<Product, DatabaseError> {
        Ok(query_as!(
            Product, 
            "INSERT INTO product (name, description, stock, price) VALUES ($1, $2, $3, $4) RETURNING *",
            self.name, self.description, self.stock, self.price
        ).fetch_one(db_client).await?)
    }
}

impl Product {
    /// Select a `Product` from the database by its ID.
    pub async fn select_one(id: i64, db_client: &ConnectionPool) -> Result<Option<Self>, DatabaseError> {
        Ok(query_as!(Self, "SELECT * FROM product WHERE id = $1", id)
            .fetch_optional(db_client)
            .await?)
    }
    /// Retrieve all `Product`s stored in the database.
    pub async fn select_all(db_client: &ConnectionPool) -> Result<Vec<Self>, DatabaseError> {
        Ok(query_as!(Self, "SELECT * FROM product").fetch_all(db_client).await?)
    }
    /// Set the count of this product in stock.
    pub fn set_stock(&mut self, stock: u32) {
        self.stock = i64::from(stock);
    }
    /// Set the product's price in pennies (GBP).
    pub fn set_price(&mut self, price: u32) {
        self.price = i64::from(price);
    }
    /// Get the count of this product in stock.
    pub fn stock(&self) -> u32 {
        u32::try_from(self.stock).expect("Stock value in database is out of allowed range")
    }
    /// Get the price of this product in pennies (GBP).
    pub fn price(&self) -> u32 {
        u32::try_from(self.price).expect("Price value in database is out of allowed range")
    }
    /// Get this product's ID primary key.
    pub const fn id(&self) -> i64 {
        self.id
    }
    /// Update the corresponding database record to match this model's state.
    pub async fn update(&self, db_client: &ConnectionPool) -> Result<(), DatabaseError> {
        Ok(query!(
            "UPDATE product SET name = $1, description = $2, stock = $3, price = $4 WHERE id = $5", 
            self.name, self.description, self.stock, self.price, self.id
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
}
