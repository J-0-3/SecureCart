//! Functions for dealing with/storing/querying products.
use serde::Deserialize;

use crate::db::{
    self,
    models::product::{Product, ProductInsert},
};

/// The parameters for a search over stored products. Any/all of the included
/// parameters can be set.
#[derive(Deserialize)]
pub struct ProductSearchParameters {
    /// The name to search for. Will match any product starting with this.
    name: Option<String>,
    /// The minimum price bound. Will match only products which cost more than this.
    price_min: Option<u32>,
    /// The maximum price bound. Will match only products which cost less than this.
    price_max: Option<u32>,
}

/// Search products stored in the database.
pub async fn search_products(
    db_conn: &db::ConnectionPool,
    params: &ProductSearchParameters,
) -> Result<Vec<Product>, db::errors::DatabaseError> {
    let mut result = Vec::new();
    // We clone search_name here to avoid cloning the entire struct.
    let search_name = params.name.clone();
    let search_price_min = params.price_min;
    let search_price_max = params.price_max;
    for product in Product::select_all(db_conn).await? {
        let name_match = search_name
            .as_ref()
            .is_none_or(|name| product.name.starts_with(name));
        let price_min_match = search_price_min
            .as_ref()
            .is_none_or(|price| product.price() >= *price);
        let price_max_match = search_price_max
            .as_ref()
            .is_none_or(|price| product.price() <= *price);
        if name_match && price_min_match && price_max_match {
            result.push(product);
        }
    }
    Ok(result)
}

/// Retrieve a specific product.
pub async fn retrieve_product(
    id: u32,
    db_conn: &db::ConnectionPool,
) -> Result<Option<Product>, db::errors::DatabaseError> {
    Product::select_one(id, db_conn).await
}

/// Retrieve only those products which are marked as listed.
pub async fn retrieve_listed_products(
    db_conn: &db::ConnectionPool,
) -> Result<Vec<Product>, db::errors::DatabaseError> {
    Ok(Product::select_all(db_conn)
        .await?
        .into_iter()
        .filter(Product::is_listed)
        .collect())
}

/// UPDATE model for a product. All fields are optional, so an empty JSON
/// object, a fully defined new Product model, or anything in between is
/// valid and only the set fields will be updated.
#[derive(Deserialize)]
pub struct ProductUpdate {
    /// The product's new name.
    name: Option<String>,
    /// The product's new price.
    price: Option<u32>,
    /// A change to the product's listing status.
    listed: Option<bool>,
    /// The product's new description.
    description: Option<String>,
}

/// Update an an existing stored product.
pub async fn update_product(
    id: u32,
    product_info: ProductUpdate,
    db_conn: &db::ConnectionPool,
) -> Result<(), errors::ProductUpdateError> {
    let mut product = Product::select_one(id, db_conn)
        .await?
        .ok_or(errors::ProductUpdateError::NonExistent)?;
    if let Some(name) = product_info.name {
        product.set_name(&name);
    }
    if let Some(price) = product_info.price {
        product.set_price(price);
    }
    if let Some(listed) = product_info.listed {
        if listed {
            product.list();
        } else {
            product.unlist();
        }
    }
    if let Some(description) = product_info.description {
        product.set_description(&description);
    }
    Ok(product.update(db_conn).await?)
}

/// Create a new product in the database.
pub async fn create_product(
    data: ProductInsert,
    db_conn: &db::ConnectionPool,
) -> Result<Product, db::errors::DatabaseError> {
    data.store(db_conn).await
}

/// Delete a given product from the database.
pub async fn delete_product(
    id: u32,
    db_conn: &db::ConnectionPool,
) -> Result<(), errors::ProductDeleteError> {
    let product = Product::select_one(id, db_conn)
        .await?
        .ok_or(errors::ProductDeleteError::NonExistent)?;
    Ok(product.delete(db_conn).await?)
}

/// Errors which can be returned by functions in this service.
pub mod errors {
    use crate::db::errors::DatabaseError;
    use thiserror::Error;

    /// Errors returned when updating products.
    #[derive(Error, Debug)]
    pub enum ProductUpdateError {
        /// Error passed up from the database storage layer.
        #[error(transparent)]
        DatabaseError(#[from] DatabaseError),
        /// Raised when the product being updated does not exist.
        #[error("The product being updated does not exist.")]
        NonExistent,
    }
    /// Errors returned when deleting products.
    #[derive(Error, Debug)]
    pub enum ProductDeleteError {
        /// Error passed up from the database storage layer.
        #[error(transparent)]
        DatabaseError(#[from] DatabaseError),
        /// Raised when the product being deleted does not exist.
        #[error("The product being deleted does not exist.")]
        NonExistent,
    }
}
