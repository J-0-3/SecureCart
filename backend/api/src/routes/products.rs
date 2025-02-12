//! Routes for CRUD operations on products.
use axum::{
    extract::{Path, Query, State},
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
        .layer(from_fn_with_state(
            state.clone(),
            session_middleware::<GenericAuthenticatedSession>,
        ));
    let admin_authenticated = Router::new()
        .route("/", post(create_product))
        .route("/{product_id}", put(update_product))
        .route("/{product_id}", delete(delete_product))
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
            products::retrieve_products::<{ ProductVisibilityScope::LISTED_ONLY }>(&state.db_conn)
                .await?
        }
        GenericAuthenticatedSession::Administrator(_) => {
            products::retrieve_products::<{ ProductVisibilityScope::INCLUDE_UNLISTED }>(
                &state.db_conn,
            )
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
            products::search_products::<{ ProductVisibilityScope::LISTED_ONLY }>(
                &state.db_conn,
                &params,
            )
            .await?
        }
        GenericAuthenticatedSession::Administrator(_) => {
            products::search_products::<{ ProductVisibilityScope::INCLUDE_UNLISTED }>(
                &state.db_conn,
                &params,
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
                product_id,
                &state.db_conn,
            )
            .await?
        }
        GenericAuthenticatedSession::Administrator(_) => {
            products::retrieve_product::<{ ProductVisibilityScope::INCLUDE_UNLISTED }>(
                product_id,
                &state.db_conn,
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
    Ok(Json(products::create_product(body, &state.db_conn).await?))
}

/// Delete a product.
async fn delete_product(
    State(state): State<AppState>,
    Path(product_id): Path<u32>,
) -> Result<(), StatusCode> {
    Ok(products::delete_product(product_id, &state.db_conn).await?)
}

/// Update a product.
async fn update_product(
    State(state): State<AppState>,
    Path(product_id): Path<u32>,
    Json(body): Json<ProductUpdate>,
) -> Result<(), StatusCode> {
    Ok(products::update_product(product_id, body, &state.db_conn).await?)
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
