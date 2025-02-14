//! Functions for dealing with/storing/querying products.
#[expect(
    clippy::useless_attribute,
    reason = "Lint is enabled only in clippy::restrictions"
)]
#[expect(
    clippy::std_instead_of_alloc,
    reason = "Does not work outside of no_std"
)]
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
    Product::search(
        &db::models::product::ProductSearchParameters {
            listed: (VISIBILITY_SCOPE == ProductVisibilityScope::LISTED_ONLY).then_some(true),
            ..Default::default()
        },
        db_conn,
    )
    .await
}

/// The parameters for a search over stored products. Any/all of the included
/// parameters can be set. This is a subset of the options available in
/// `db::models::product::ProductSearchParameters` which are settable by
/// external callers.
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
    Product::search(
        &db::models::product::ProductSearchParameters {
            name: params.name.clone(),
            price_min: params.price_min,
            price_max: params.price_max,
            listed: (VISIBILITY_SCOPE == ProductVisibilityScope::LISTED_ONLY).then_some(true),
        },
        db_conn,
    )
    .await
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

/// Add an image to a product, returning the path (URI) at which the image can be
/// found.
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

/// List the paths (URIs) of all images associated with the given product.
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

/// Delete an image from a product at a given path.
pub async fn delete_image(
    product_id: u32,
    path: &str,
    db_conn: &db::ConnectionPool,
) -> Result<(), errors::ImageDeleteError> {
    // This removes the S3 URI and bucket if present, and ensures that the path
    // starts with exactly one leading separator (as if relative to the bucket root).
    let mut normalised_path = String::from("/");
    normalised_path.push_str(
        path.trim_start_matches(&*S3_EXTERNAL_URI)
            .trim_start_matches('/')
            .trim_start_matches(&*S3_BUCKET)
            .trim_start_matches('/'),
    );
    let product = ProductImage::select(product_id, &normalised_path, db_conn)
        .await?
        .ok_or(errors::ImageDeleteError::NonExistentImage)?;
    product.delete(db_conn).await?;
    Ok(())
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
    /// Errors returned when adding images to products.
    #[derive(Error, Debug)]
    pub enum AddImageError {
        /// Error passed up from the database storage layer.
        #[error(transparent)]
        DatabaseError(#[from] DatabaseError),
        /// Error passed up from the media storage layer.
        #[error(transparent)]
        MediaStoreError(#[from] StoreImageError),
        /// Raised when the product in question does not exist.
        #[error("The product being added to does not exist.")]
        NonExistent,
    }
    /// Errors returned when deleting images from products.
    #[derive(Error, Debug)]
    pub enum ImageDeleteError {
        /// Error passed up from the database storage layer.
        #[error(transparent)]
        DatabaseError(#[from] DatabaseError),
        /// Raised when the image being deleted does not exist.
        #[error("The image being deleted does not exist")]
        NonExistentImage,
    }
}
