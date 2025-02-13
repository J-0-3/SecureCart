//! Functions for dealing with/storing/querying products.
use std::sync::Arc;

use object_store::ObjectStore;
use serde::Deserialize;

use crate::{
    constants::s3::{S3_BUCKET, S3_EXTERNAL_URI},
    db::{
        self,
        models::{
            product::{Product, ProductInsert},
            product_image::{ProductImage, ProductImageInsert},
        },
    },
};

use super::media;

// This is a little weird and unpleasant (implementing an enum manually),
// but it is necessary since enums are non-const and not allowed as const
// generic parameters. If enums become supported as const generic parameters,
// one should immediately be used here instead.

/// The type used to represent product's visibility scopes (since enums are non-const)
type ProductVisibilityScopeT = bool;

/// Possible visibility scopes for querying products.
#[expect(non_snake_case, reason = "This is acting as a poor-man's enum")]
pub mod ProductVisibilityScope {
    /// Include only publically listed products.
    pub const LISTED_ONLY: super::ProductVisibilityScopeT = false;
    /// Include all products, whether listed or not.
    pub const INCLUDE_UNLISTED: super::ProductVisibilityScopeT = true;
}

/// Retrieve a specific product. Generically parameterised over the visibility
/// scope to retrieve from. `VISIBILITY_SCOPE` must *ONLY* be set to a value from
/// `ProductVisibilityScope`, or the function's behaviour is undefined.
pub async fn retrieve_product<const VISIBILITY_SCOPE: ProductVisibilityScopeT>(
    id: u32,
    db_conn: &db::ConnectionPool,
) -> Result<Option<Product>, db::errors::DatabaseError> {
    Ok(Product::select_one(id, db_conn).await?.filter(|prod| {
        VISIBILITY_SCOPE == ProductVisibilityScope::INCLUDE_UNLISTED || prod.is_listed()
    }))
}

/// List all products in the database. Generically parameterised over the visibility
/// scope to retrieve from. `VISIBILITY_SCOPE` must *ONLY* be set to a value from
/// `ProductVisibilityScope`, or the function's behaviour is undefined.
pub async fn retrieve_products<const VISIBILITY_SCOPE: ProductVisibilityScopeT>(
    db_conn: &db::ConnectionPool,
) -> Result<Vec<Product>, db::errors::DatabaseError> {
    Ok(Product::select_all(db_conn)
        .await?
        .into_iter()
        .filter(|prod| {
            VISIBILITY_SCOPE == ProductVisibilityScope::INCLUDE_UNLISTED || prod.is_listed()
        })
        .collect())
}

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

/// Search products stored in the database. Generically parameterised over the visibility
/// scope to retrieve from. `VISIBILITY_SCOPE` must *ONLY* be set to a value from
/// `ProductVisibilityScope`, or the function's behaviour is undefined.
pub async fn search_products<const VISIBILITY_SCOPE: ProductVisibilityScopeT>(
    db_conn: &db::ConnectionPool,
    params: &ProductSearchParameters,
) -> Result<Vec<Product>, db::errors::DatabaseError> {
    let mut result = Vec::new();
    // We clone search_name here to avoid cloning the entire struct.
    let search_name = params.name.clone();
    let search_price_min = params.price_min;
    let search_price_max = params.price_max;
    for product in retrieve_products::<VISIBILITY_SCOPE>(db_conn).await? {
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

pub async fn add_image(
    product_id: u32,
    image: Vec<u8>,
    db_conn: &db::ConnectionPool,
    media_store: Arc<dyn ObjectStore>,
) -> Result<String, errors::AddImageError> {
    let _: Product = Product::select_one(product_id, db_conn)
        .await?
        .ok_or(errors::AddImageError::NonExistent)?;
    let image_path = media::store_image(media_store, image).await?;
    let image_insert = ProductImageInsert::new(product_id, &image_path);
    let _: ProductImage = image_insert.store(db_conn).await?;
    Ok(format!(
        "{}/{}/{}",
        &*S3_EXTERNAL_URI,
        &*S3_BUCKET,
        image_path.trim_start_matches('/')
    ))
}

pub async fn list_images(
    product_id: u32,
    db_conn: &db::ConnectionPool,
) -> Result<Vec<String>, db::errors::DatabaseError> {
    Ok(ProductImage::select_all(product_id, db_conn)
        .await?
        .into_iter()
        .map(|img| {
            format!(
                "{}/{}/{}",
                &*S3_EXTERNAL_URI,
                &*S3_BUCKET,
                img.path.trim_start_matches('/')
            )
        })
        .collect())
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
    use crate::services::media::errors::StoreImageError;
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
    #[derive(Error, Debug)]
    pub enum AddImageError {
        #[error(transparent)]
        DatabaseError(#[from] DatabaseError),
        #[error(transparent)]
        MediaStoreError(#[from] StoreImageError),
        #[error("The product being added to does not exist.")]
        NonExistent,
    }
}
