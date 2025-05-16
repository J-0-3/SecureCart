//! Routes for handling order creation and access, interacts with the order service
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    middleware::from_fn_with_state,
    routing::{delete, get, post},
    Extension, Json, Router,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    constants::api::API_URI_PREFIX,
    db::models::apporder::{AppOrder, AppOrderSearchParameters},
    middleware::session::session_middleware,
    services::{
        orders::{self},
        sessions::{AdministratorSession, CustomerSession, GenericAuthenticatedSession},
    },
    state::AppState,
    utils::httperror::HttpError,
};

/// TODO: add documentation
pub fn create_router(state: &AppState) -> Router<AppState> {
    let customer = Router::new()
        .route("/", post(create_order))
        .layer(from_fn_with_state(
            state.clone(),
            session_middleware::<CustomerSession>,
        ));
    let administrator = Router::new()
        .route("/{order_id}/fulfil", post(fulfil_order))
        .layer(from_fn_with_state(
            state.clone(),
            session_middleware::<AdministratorSession>,
        ));
    let authenticated = Router::new()
        .route("/", get(search_orders))
        .route("/{order_id}", get(retrieve_order))
        .route("/{order_id}", delete(delete_order))
        .layer(from_fn_with_state(
            state.clone(),
            session_middleware::<GenericAuthenticatedSession>,
        ));
    customer.merge(administrator).merge(authenticated)
}

#[derive(Deserialize)]
/// TODO: add documentation
struct CreateOrderRequest {
    /// TODO: add documentation
    products: Vec<CreateOrderRequestProductEntry>,
}

#[derive(Deserialize)]
/// TODO: add documentation
struct CreateOrderRequestProductEntry {
    /// TODO: add documentation
    product: Uuid,
    /// TODO: add documentation
    count: u32,
}

/// TODO: add documentation
async fn create_order(
    State(state): State<AppState>,
    Extension(session): Extension<CustomerSession>,
    Json(body): Json<CreateOrderRequest>,
) -> Result<Json<AppOrder>, HttpError> {
    let user_id = session.user_id();
    Ok(Json(
        orders::create_order(
            user_id,
            body.products
                .into_iter()
                .map(|entry| (entry.product, entry.count))
                .collect(),
            &state.db,
        )
        .await?,
    ))
}

#[derive(Serialize)]
/// TODO: add documentation
struct OrderSearchResponse {
    /// TODO: add documentation
    orders: Vec<AppOrder>,
}

async fn search_orders(
    State(state): State<AppState>,
    Extension(session): Extension<GenericAuthenticatedSession>,
    Query(params): Query<AppOrderSearchParameters>,
) -> Result<Json<OrderSearchResponse>, HttpError> {
    Ok(Json(OrderSearchResponse {
        orders: match session {
            GenericAuthenticatedSession::Customer(customer_session) => {
                orders::search_orders(
                    AppOrderSearchParameters {
                        user_id: Some(customer_session.user_id()),
                        status: params.status,
                    },
                    &state.db,
                )
                .await?
            }
            GenericAuthenticatedSession::Administrator(_) => {
                orders::search_orders(params, &state.db).await?
            }
        },
    }))
}

#[derive(Serialize)]
/// TODO: add documentation
struct RetrieveOrderResponse {
    /// TODO: add documentation
    order: AppOrder,
    /// TODO: add documentation
    items: Vec<(String, u32)>,
}

/// TODO: add documentation
async fn retrieve_order(
    State(state): State<AppState>,
    Extension(session): Extension<GenericAuthenticatedSession>,
    Path(order_id): Path<Uuid>,
) -> Result<Json<RetrieveOrderResponse>, HttpError> {
    let maybe_order = orders::get_order_with_items(order_id, &state.db)
        .await?
        .map(|order| RetrieveOrderResponse {
            order: order.order,
            items: order
                .items
                .iter()
                .map(|&(product_id, count)| {
                    (format!("{}/products/{product_id}", *API_URI_PREFIX), count)
                })
                .collect(),
        });
    let order = match session {
        GenericAuthenticatedSession::Administrator(_) => maybe_order.map_or_else(
            || {
                eprintln!("Administrator request to view order {order_id}, which does not exist.");
                Err(HttpError::new(
                    StatusCode::NOT_FOUND,
                    Some(format!("Order {order_id} not found")),
                ))
            },
            Ok,
        ),
        GenericAuthenticatedSession::Customer(customer) => {
            match maybe_order {
                None => {
                    eprintln!("Customer with ID {} attempted to view order {order_id}, which does not exist.", customer.user_id());
                    Err(StatusCode::FORBIDDEN.into()) // 401 not 404 to prevent enumerating valid order IDs.
                }
                Some(order) => {
                    if order.order.user_id() == customer.user_id() {
                        Ok(order)
                    } else {
                        eprintln!(
                            "User {} attempted to view order {} owned by {}.",
                            customer.user_id(),
                            order_id,
                            order.order.user_id()
                        );
                        Err(StatusCode::FORBIDDEN.into())
                    }
                }
            }
        }
    }?;
    Ok(Json(order))
}

/// TODO: add documentation
async fn delete_order(
    State(state): State<AppState>,
    Extension(session): Extension<GenericAuthenticatedSession>,
    Path(order_id): Path<Uuid>,
) -> Result<(), HttpError> {
    if let GenericAuthenticatedSession::Customer(customer_session) = session {
        let user_id = customer_session.user_id();
        let order = orders::get_order(order_id, &state.db)
            .await?
            .ok_or_else(|| {
                eprintln!("Attempted to delete order which does not exist while authenticated as user {user_id}");
                StatusCode::FORBIDDEN // 401 not 404 to obscure whether this order ID is valid or
                                      // for another user or not.
            })?;
        let order_owner = order.user_id();
        if user_id != order_owner {
            eprintln!(
                "User {user_id} attempted to delete order {order_id} owned by {order_owner}."
            );
            return Err(StatusCode::FORBIDDEN.into());
        }
    }
    orders::delete_order(order_id, &state.db).await?;
    Ok(())
}

/// TODO: add documentation
async fn fulfil_order(
    State(state): State<AppState>,
    Path(order_id): Path<Uuid>,
) -> Result<(), HttpError> {
    orders::fulfil_order(order_id, &state.db).await?;
    Ok(())
}

impl From<orders::errors::OrderCreationError> for HttpError {
    fn from(error: orders::errors::OrderCreationError) -> Self {
        match error {
            orders::errors::OrderCreationError::DatabaseError(err) => err.into(),
            orders::errors::OrderCreationError::UserNonExistent(user_id) => {
                eprintln!("Attempted to create an order while authenticated as user {user_id} who does not exist.");
                Self::from(StatusCode::UNAUTHORIZED)
            }
            orders::errors::OrderCreationError::ProductNonExistent(product_id) => {
                eprintln!(
                    "Attempted to create an order containing product {product_id} which does not exist."
                );
                Self::new(
                    StatusCode::NOT_FOUND,
                    Some(format!("Product {product_id} not found")),
                )
            }
            orders::errors::OrderCreationError::CostTooLarge => {
                eprintln!("Order total cost exceeded i64 max");
                Self::new(
                    StatusCode::BAD_REQUEST,
                    Some(String::from("Order total exceeded max allowable value")),
                )
            }
        }
    }
}

impl From<orders::errors::OrderDeletionError> for HttpError {
    fn from(error: orders::errors::OrderDeletionError) -> Self {
        match error {
            orders::errors::OrderDeletionError::DatabaseError(err) => err.into(),
            orders::errors::OrderDeletionError::OrderNonExistent(order_id) => {
                eprintln!("Attempted to delete order {order_id}, which does not exist.");
                Self::new(
                    StatusCode::NOT_FOUND,
                    Some(format!("Order {order_id} not found")),
                )
            }
        }
    }
}

impl From<orders::errors::OrderFulfilmentError> for HttpError {
    fn from(error: orders::errors::OrderFulfilmentError) -> Self {
        match error {
            orders::errors::OrderFulfilmentError::DatabaseError(err) => err.into(),
            orders::errors::OrderFulfilmentError::OrderNonExistent(order_id) => {
                eprintln!("Attempted to delete a non-existent order.");
                Self::new(
                    StatusCode::NOT_FOUND,
                    Some(format!("Order {order_id} not found")),
                )
            }
            orders::errors::OrderFulfilmentError::OrderNotConfirmed(order_id) => {
                eprintln!("Attempted to fulfil order {order_id} which is not yet confirmed.");
                Self::new(
                    StatusCode::BAD_REQUEST,
                    Some(String::from("Order is not confirmed")),
                )
            }
        }
    }
}
