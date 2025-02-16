use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    middleware::from_fn_with_state,
    routing::{delete, get, post},
    Extension, Json, Router,
};
use serde::{Deserialize, Serialize};

use crate::{
    db::models::apporder::AppOrder,
    middleware::auth::session_middleware,
    services::{
        orders::{self, AppOrderWithItems},
        sessions::{AdministratorSession, CustomerSession, GenericAuthenticatedSession},
    },
    state::AppState,
};

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
struct CreateOrderRequestProductEntry {
    product: u32,
    count: u32,
}

#[derive(Deserialize)]
struct CreateOrderRequest {
    products: Vec<CreateOrderRequestProductEntry>,
}

#[derive(Serialize)]
struct CreateOrderResponse {
    order: AppOrder,
    payment_intent: String,
}

async fn create_order(
    State(state): State<AppState>,
    Extension(session): Extension<CustomerSession>,
    Json(body): Json<CreateOrderRequest>,
) -> Result<Json<AppOrder>, StatusCode> {
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

#[derive(Deserialize)]
struct OrdersQueryParams {
    user_id: Option<u32>,
}

#[derive(Serialize)]
struct OrderSearchResponse {
    orders: Vec<AppOrder>,
}

async fn search_orders(
    State(state): State<AppState>,
    Extension(session): Extension<GenericAuthenticatedSession>,
    Query(params): Query<OrdersQueryParams>,
) -> Result<Json<OrderSearchResponse>, StatusCode> {
    Ok(Json(OrderSearchResponse {
        orders: match session {
            GenericAuthenticatedSession::Customer(customer_session) => {
                orders::search_orders(customer_session.user_id(), &state.db).await?
            }
            GenericAuthenticatedSession::Administrator(_) => match params.user_id {
                Some(user_id) => orders::search_orders(user_id, &state.db).await?,
                None => orders::list_orders(&state.db).await?,
            },
        },
    }))
}

async fn retrieve_order(
    State(state): State<AppState>,
    Extension(session): Extension<GenericAuthenticatedSession>,
    Path(order_id): Path<u32>,
) -> Result<Json<AppOrderWithItems>, StatusCode> {
    let maybe_order = orders::get_order_with_items(order_id, &state.db).await?;
    let order = match session {
        GenericAuthenticatedSession::Administrator(_) => maybe_order.map_or_else(
            || {
                eprintln!("Administrator request to view order {order_id}, which does not exist.");
                Err(StatusCode::NOT_FOUND)
            },
            Ok,
        ),
        GenericAuthenticatedSession::Customer(customer) => {
            match maybe_order {
                None => {
                    eprintln!("Customer with ID {} attempted to view order {order_id}, which does not exist.", customer.user_id());
                    Err(StatusCode::FORBIDDEN) // 401 not 404 to prevent enumerating valid order IDs.
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
                        Err(StatusCode::FORBIDDEN)
                    }
                }
            }
        }
    }?;
    Ok(Json(order))
}

async fn delete_order(
    State(state): State<AppState>,
    Extension(session): Extension<GenericAuthenticatedSession>,
    Path(order_id): Path<u32>,
) -> Result<(), StatusCode> {
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
            return Err(StatusCode::FORBIDDEN);
        }
    }
    orders::delete_order(order_id, &state.db).await?;
    Ok(())
}

async fn fulfil_order(
    State(state): State<AppState>,
    Path(order_id): Path<u32>,
) -> Result<(), StatusCode> {
    orders::fulfil_order(order_id, &state.db).await?;
    Ok(())
}

impl From<orders::errors::OrderCreationError> for StatusCode {
    fn from(error: orders::errors::OrderCreationError) -> Self {
        match error {
            orders::errors::OrderCreationError::DatabaseError(err) => err.into(),
            orders::errors::OrderCreationError::UserNonExistent => {
                eprintln!("Attempted to create an order while authenticated as a user who does not exist.");
                Self::UNAUTHORIZED
            }
            orders::errors::OrderCreationError::ProductNonExistent => {
                eprintln!(
                    "Attempted to create an order containing a product which does not exist."
                );
                Self::NOT_FOUND
            }
            orders::errors::OrderCreationError::CostTooLarge => {
                eprintln!("Order total cost exceeded i64 max");
                Self::BAD_REQUEST
            }
        }
    }
}

impl From<orders::errors::OrderDeletionError> for StatusCode {
    fn from(error: orders::errors::OrderDeletionError) -> Self {
        match error {
            orders::errors::OrderDeletionError::DatabaseError(err) => err.into(),
            orders::errors::OrderDeletionError::OrderNonExistent => {
                eprintln!("Attempted to delete a non-existent order.");
                Self::NOT_FOUND
            }
        }
    }
}

impl From<orders::errors::OrderFulfilmentError> for StatusCode {
    fn from(error: orders::errors::OrderFulfilmentError) -> Self {
        match error {
            orders::errors::OrderFulfilmentError::DatabaseError(err) => err.into(),
            orders::errors::OrderFulfilmentError::OrderNonExistent => {
                eprintln!("Attempted to delete a non-existent order.");
                Self::NOT_FOUND
            }
        }
    }
}
