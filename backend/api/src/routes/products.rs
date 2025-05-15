//! Routes for CRUD operations on products.
use axum::{
    extract::{Multipart, Path, Query, State},
    http::StatusCode,
    middleware::from_fn_with_state,
    routing::{delete, get, post, put},
    Extension, Json, Router,
};
use serde::Serialize;
use uuid::Uuid;

use crate::{
    db::models::product::{Product, ProductInsert},
    middleware::session::session_middleware,
    services::{
        products::{self, ProductSearchParameters, ProductUpdate, ProductVisibilityScope},
        sessions::{AdministratorSession, GenericAuthenticatedSession},
    },
    state::AppState,
    utils::httperror::HttpError,
};

/// Create a router for routes under the product service.
pub fn create_router(state: &AppState) -> Router<AppState> {
    let authenticated = Router::new()
        .route("/", get(search_products))
        .route("/{product_id}", get(get_product))
        .route("/{product_id}/images", get(list_product_images))
        .layer(from_fn_with_state(
            state.clone(),
            session_middleware::<GenericAuthenticatedSession>,
        ));
    let admin_authenticated = Router::new()
        .route("/", post(create_product))
        .route("/{product_id}", put(update_product))
        .route("/{product_id}", delete(delete_product))
        .route("/{product_id}/images", post(add_product_image))
        .route("/{product_id}/images/{path}", delete(delete_product_image))
        .layer(from_fn_with_state(
            state.clone(),
            session_middleware::<AdministratorSession>,
        ));
    authenticated.merge(admin_authenticated)
}

/// The response to /products or /products/search.
#[derive(Serialize)]
struct ListProductsResponse {
    /// The products returned by the query.
    products: Vec<Product>,
}

/// Search for matching products.
async fn search_products(
    State(state): State<AppState>,
    Extension(session): Extension<GenericAuthenticatedSession>,
    Query(params): Query<ProductSearchParameters>,
) -> Result<Json<ListProductsResponse>, HttpError> {
    let products = match session {
        GenericAuthenticatedSession::Customer(_) => {
            products::search_products::<{ ProductVisibilityScope::LISTED_ONLY }>(&state.db, &params)
                .await?
        }
        GenericAuthenticatedSession::Administrator(_) => {
            products::search_products::<{ ProductVisibilityScope::INCLUDE_UNLISTED }>(
                &state.db, &params,
            )
            .await?
        }
    };
    Ok(Json(ListProductsResponse { products }))
}

/// Get a product by its ID.
async fn get_product(
    State(state): State<AppState>,
    Extension(session): Extension<GenericAuthenticatedSession>,
    Path(product_id): Path<Uuid>,
) -> Result<Json<Product>, HttpError> {
    let product = match session {
        GenericAuthenticatedSession::Customer(_) => {
            products::retrieve_product::<{ ProductVisibilityScope::LISTED_ONLY }>(
                product_id, &state.db,
            )
            .await?
        }
        GenericAuthenticatedSession::Administrator(_) => {
            products::retrieve_product::<{ ProductVisibilityScope::INCLUDE_UNLISTED }>(
                product_id, &state.db,
            )
            .await?
        }
    };
    Ok(Json(product.ok_or(StatusCode::NOT_FOUND)?))
}

/// Create a new product.
async fn create_product(
    State(state): State<AppState>,
    Json(body): Json<ProductInsert>,
) -> Result<Json<Product>, HttpError> {
    Ok(Json(products::create_product(body, &state.db).await?))
}

/// Delete a product.
async fn delete_product(
    State(state): State<AppState>,
    Path(product_id): Path<Uuid>,
) -> Result<(), HttpError> {
    Ok(products::delete_product(product_id, &state.db).await?)
}

/// Update a product.
async fn update_product(
    State(state): State<AppState>,
    Path(product_id): Path<Uuid>,
    Json(body): Json<ProductUpdate>,
) -> Result<(), HttpError> {
    Ok(products::update_product(product_id, body, &state.db).await?)
}

/// The response to POST /products/{id}/images.
#[derive(Serialize)]
struct AddImageResponse {
    /// The path where the uploaded image was stored.
    path: String,
}

/// Add an image to a given product. This, unlike most endpoints, accepts
/// multipart form data instead of JSON. This is because that is the most
/// natural way to do a file upload over HTTP.
async fn add_product_image(
    State(state): State<AppState>,
    Path(product_id): Path<Uuid>,
    mut data: Multipart,
) -> Result<Json<AddImageResponse>, HttpError> {
    loop {
        let Some(field) = data.next_field().await.map_err(|err| {
            eprintln!("Error while processing multipart data: {err}");
            StatusCode::UNPROCESSABLE_ENTITY
        })?
        else {
            eprintln!("Image was not included in multipart form data.");
            return Err(HttpError::new(
                StatusCode::UNPROCESSABLE_ENTITY,
                Some(String::from("Image field is missing from form data")),
            ));
        };
        if field.name().ok_or_else(|| {
            eprintln!("Multipart field missing name in request to add image");
            HttpError::new(
                StatusCode::UNPROCESSABLE_ENTITY,
                Some(String::from("A multipart form field is missing a name")),
            )
        })? == "image"
        {
            let result = products::add_image(
                product_id,
                field
                    .bytes()
                    .await
                    .map_err(|err| {
                        eprintln!("Multipart form image data unprocessable: {err}");
                        HttpError::new(StatusCode::UNPROCESSABLE_ENTITY, Some(err.to_string()))
                    })?
                    .to_vec(),
                &state.db,
                state.media_store,
            )
            .await?;
            break Ok(Json(AddImageResponse { path: result }));
        }
    }
}

/// Delete (disassociate) an image from a product.
async fn delete_product_image(
    State(state): State<AppState>,
    Path((product_id, path)): Path<(Uuid, String)>,
) -> Result<(), HttpError> {
    Ok(products::delete_image(product_id, &path, &state.db).await?)
}

/// The response to /product/{id}/images
#[derive(Serialize)]
struct ListImagesResponse {
    /// The list of images returned.
    images: Vec<String>,
}

/// List URIs for all images associated with a product.
async fn list_product_images(
    State(state): State<AppState>,
    Path(product_id): Path<Uuid>,
) -> Result<Json<ListImagesResponse>, HttpError> {
    Ok(Json(
        products::list_images(product_id, &state.db)
            .await
            .map(|images| ListImagesResponse { images })?,
    ))
}

impl From<products::errors::ProductDeleteError> for HttpError {
    fn from(err: products::errors::ProductDeleteError) -> Self {
        match err {
            products::errors::ProductDeleteError::DatabaseError(error) => error.into(),
            products::errors::ProductDeleteError::NonExistent(product_id) => {
                eprintln!("Attempted to delete product {product_id}, which does not exist");
                Self::new(
                    StatusCode::NOT_FOUND,
                    Some(format!("Product {product_id} not found")),
                )
            }
        }
    }
}

impl From<products::errors::ProductUpdateError> for HttpError {
    fn from(err: products::errors::ProductUpdateError) -> Self {
        match err {
            products::errors::ProductUpdateError::DatabaseError(error) => error.into(),
            products::errors::ProductUpdateError::NonExistent(product_id) => {
                eprintln!("Attempted to update product {product_id}, which does not exist");
                Self::new(
                    StatusCode::NOT_FOUND,
                    Some(format!("Product {product_id} not found")),
                )
            }
        }
    }
}

impl From<products::errors::AddImageError> for HttpError {
    fn from(err: products::errors::AddImageError) -> Self {
        match err {
            products::errors::AddImageError::DatabaseError(error) => error.into(),
            products::errors::AddImageError::MediaStoreError(error) => {
                eprintln!("Error in media object store while adding image: {error}");
                Self::from(StatusCode::INTERNAL_SERVER_ERROR)
            }
            products::errors::AddImageError::NonExistent(product_id) => {
                eprintln!("Attempted to add an image to product {product_id} which does not exist");
                Self::new(
                    StatusCode::NOT_FOUND,
                    Some(format!("Product {product_id} not found.")),
                )
            }
        }
    }
}

impl From<products::errors::ImageDeleteError> for HttpError {
    fn from(err: products::errors::ImageDeleteError) -> Self {
        match err {
            products::errors::ImageDeleteError::DatabaseError(error) => error.into(),
            products::errors::ImageDeleteError::NonExistentImage(path, product_id) => {
                eprintln!(
                    "Attempted to delete non-existent image at {path} from product {product_id}"
                );
                Self::new(
                    StatusCode::NOT_FOUND,
                    Some(format!("Image {path} not found on product {product_id}")),
                )
            }
        }
    }
}
