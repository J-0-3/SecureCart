//! Routes for CRUD operations on products.
use axum::{
    extract::{Multipart, Path, Query, State},
    http::StatusCode,
    middleware::from_fn_with_state,
    routing::{delete, get, post, put},
    Extension, Json, Router,
};
use serde::Serialize;

use crate::{
    db::{
        errors::DatabaseError,
        models::product::{Product, ProductInsert},
    },
    middleware::auth::session_middleware,
    services::{
        products::{self, ProductSearchParameters, ProductUpdate, ProductVisibilityScope},
        sessions::{AdministratorSession, GenericAuthenticatedSession},
    },
    state::AppState,
};

/// Create a router for routes under the product service.
pub fn create_router(state: &AppState) -> Router<AppState> {
    let unauthenticated = Router::new().route("/", get(root));
    let authenticated = Router::new()
        .route("/all", get(list_products))
        .route("/search", get(search_products))
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
        .layer(from_fn_with_state(
            state.clone(),
            session_middleware::<AdministratorSession>,
        ));
    unauthenticated
        .merge(authenticated)
        .merge(admin_authenticated)
}

/// Simply a healthcheck that this component is functional.
async fn root() -> Json<String> {
    Json("Products service is running".to_owned())
}

/// The response to /products/all or /products/search.
#[derive(Serialize)]
struct ListProductsResponse {
    /// The products returned by the query.
    products: Vec<Product>,
}

/// List all listed stored products.
async fn list_products(
    State(state): State<AppState>,
    Extension(session): Extension<GenericAuthenticatedSession>,
) -> Result<Json<ListProductsResponse>, StatusCode> {
    let products = match session {
        GenericAuthenticatedSession::Customer(_) => {
            products::retrieve_products::<{ ProductVisibilityScope::LISTED_ONLY }>(&state.db)
                .await?
        }
        GenericAuthenticatedSession::Administrator(_) => {
            products::retrieve_products::<{ ProductVisibilityScope::INCLUDE_UNLISTED }>(&state.db)
                .await?
        }
    };

    Ok(Json(ListProductsResponse { products }))
}

/// Search for matching products.
async fn search_products(
    State(state): State<AppState>,
    Extension(session): Extension<GenericAuthenticatedSession>,
    Query(params): Query<ProductSearchParameters>,
) -> Result<Json<ListProductsResponse>, StatusCode> {
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
    Path(product_id): Path<u32>,
) -> Result<Json<Product>, StatusCode> {
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
) -> Result<Json<Product>, StatusCode> {
    Ok(Json(products::create_product(body, &state.db).await?))
}

/// Delete a product.
async fn delete_product(
    State(state): State<AppState>,
    Path(product_id): Path<u32>,
) -> Result<(), StatusCode> {
    Ok(products::delete_product(product_id, &state.db).await?)
}

/// Update a product.
async fn update_product(
    State(state): State<AppState>,
    Path(product_id): Path<u32>,
    Json(body): Json<ProductUpdate>,
) -> Result<(), StatusCode> {
    Ok(products::update_product(product_id, body, &state.db).await?)
}

#[derive(Serialize)]
struct AddImageResponse {
    path: String,
}
async fn add_product_image(
    State(state): State<AppState>,
    Path(product_id): Path<u32>,
    mut data: Multipart,
) -> Result<Json<AddImageResponse>, StatusCode> {
    loop {
        let Some(field) = data.next_field().await.map_err(|err| {
            eprintln!("Error while processing multipart data: {err}");
            StatusCode::UNPROCESSABLE_ENTITY
        })?
        else {
            eprintln!("Image was not included in multipart form data.");
            return Err(StatusCode::UNPROCESSABLE_ENTITY);
        };
        if field.name().ok_or_else(|| {
            eprintln!("Multipart field missing name");
            StatusCode::UNPROCESSABLE_ENTITY
        })? == "image"
        {
            let result = products::add_image(
                product_id,
                field
                    .bytes()
                    .await
                    .map_err(|err| {
                        eprintln!("Multipart form image data unprocessable: {err}");
                        StatusCode::UNPROCESSABLE_ENTITY
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

#[derive(Serialize)]
struct ListImagesResponse {
    images: Vec<String>,
}

async fn list_product_images(
    State(state): State<AppState>,
    Path(product_id): Path<u32>,
) -> Result<Json<ListImagesResponse>, StatusCode> {
    Ok(Json(
        products::list_images(product_id, &state.db)
            .await
            .map(|images| ListImagesResponse { images })?,
    ))
}

impl From<DatabaseError> for StatusCode {
    fn from(_: DatabaseError) -> Self {
        eprintln!("Database error in product handler");
        Self::INTERNAL_SERVER_ERROR
    }
}

impl From<products::errors::ProductDeleteError> for StatusCode {
    fn from(err: products::errors::ProductDeleteError) -> Self {
        match err {
            products::errors::ProductDeleteError::DatabaseError(error) => error.into(),
            products::errors::ProductDeleteError::NonExistent => {
                eprintln!("Attempted to delete a product which does not exist");
                Self::NOT_FOUND
            }
        }
    }
}

impl From<products::errors::ProductUpdateError> for StatusCode {
    fn from(err: products::errors::ProductUpdateError) -> Self {
        match err {
            products::errors::ProductUpdateError::DatabaseError(error) => error.into(),
            products::errors::ProductUpdateError::NonExistent => {
                eprintln!("Attempted to update a product which does not exist");
                Self::NOT_FOUND
            }
        }
    }
}

impl From<products::errors::AddImageError> for StatusCode {
    fn from(err: products::errors::AddImageError) -> Self {
        match err {
            products::errors::AddImageError::DatabaseError(error) => error.into(),
            products::errors::AddImageError::MediaStoreError(error) => {
                eprintln!("Error in media object store while adding image: {error}");
                Self::INTERNAL_SERVER_ERROR
            }
            products::errors::AddImageError::NonExistent => {
                eprintln!("Attempted to add an image to a product which does not exist");
                Self::NOT_FOUND
            }
        }
    }
}
