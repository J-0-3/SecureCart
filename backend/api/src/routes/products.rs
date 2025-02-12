use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    middleware::from_fn_with_state,
    routing::{delete, get, post, put},
    Json, Router,
};
use serde::Serialize;

use crate::{
    db::models::product::{Product, ProductInsert},
    middleware::auth::session_middleware,
    services::{
        products::{self, ProductSearchParameters, ProductUpdate},
        sessions::{AdministratorSession, GenericAuthenticatedSession},
    },
    state::AppState,
};

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

async fn root() -> Json<String> {
    Json("Products service is running".to_owned())
}

#[derive(Serialize)]
struct ListProductsResponse {
    products: Vec<Product>,
}
async fn list_products(
    State(state): State<AppState>,
) -> Result<Json<ListProductsResponse>, StatusCode> {
    let products = products::retrieve_listed_products(&state.db_conn).await?;
    Ok(Json(ListProductsResponse { products }))
}

async fn search_products(
    State(state): State<AppState>,
    Query(params): Query<ProductSearchParameters>,
) -> Result<Json<ListProductsResponse>, StatusCode> {
    let products = products::search_products(&state.db_conn, &params).await?;
    Ok(Json(ListProductsResponse { products }))
}

async fn get_product(
    State(state): State<AppState>,
    Path(product_id): Path<u32>,
) -> Result<Json<Product>, StatusCode> {
    Ok(Json(
        products::retrieve_product(product_id, &state.db_conn)
            .await?
            .ok_or(StatusCode::NOT_FOUND)?,
    ))
}

async fn create_product(
    State(state): State<AppState>,
    Json(body): Json<ProductInsert>,
) -> Result<Json<Product>, StatusCode> {
    Ok(Json(products::create_product(body, &state.db_conn).await?))
}

async fn delete_product(
    State(state): State<AppState>,
    Path(product_id): Path<u32>,
) -> Result<(), StatusCode> {
    Ok(products::delete_product(product_id, &state.db_conn).await?)
}

async fn update_product(
    State(state): State<AppState>,
    Path(product_id): Path<u32>,
    Json(body): Json<ProductUpdate>,
) -> Result<(), StatusCode> {
    Ok(products::update_product(product_id, body, &state.db_conn).await?)
}

impl From<crate::db::errors::DatabaseError> for StatusCode {
    fn from(_: crate::db::errors::DatabaseError) -> Self {
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
