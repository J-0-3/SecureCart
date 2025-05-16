//! Logic for handling orders, interacts with the `AppOrder` model.
use serde::Serialize;
use time::{OffsetDateTime, PrimitiveDateTime};
use uuid::Uuid;

use crate::db::{
    self,
    models::{
        apporder::{AppOrder, AppOrderInsert, AppOrderSearchParameters, AppOrderStatus},
        appuser::AppUser,
        order_item::{OrderItem, OrderItemInsert},
        product::Product,
    },
};

/// TODO: add documentation
pub async fn confirm_order(
    order_id: Uuid,
    db_conn: &db::ConnectionPool,
) -> Result<(), errors::OrderConfirmationError> {
    let mut order = AppOrder::select_one(order_id, db_conn)
        .await?
        .ok_or(errors::OrderConfirmationError::OrderNonExistent(order_id))?;
    order.set_status(AppOrderStatus::Confirmed);
    order.update(db_conn).await?;
    Ok(())
}

#[derive(Serialize)]
/// TODO: add documentation
pub struct AppOrderWithItems {
    /// TODO: add documentation
    pub order: AppOrder,
    /// TODO: add documentation
    pub items: Vec<(Uuid, u32)>, // id, count
}

/// TODO: add documentation
pub async fn create_order(
    user_id: Uuid,
    product_counts: Vec<(Uuid, u32)>,
    db_conn: &db::ConnectionPool,
) -> Result<AppOrder, errors::OrderCreationError> {
    AppUser::select_one(user_id, db_conn)
        .await?
        .ok_or(errors::OrderCreationError::UserNonExistent(user_id))?;
    let current_time = OffsetDateTime::now_utc();
    let mut total_cost: u64 = 0;
    for &(product_id, count) in &product_counts {
        let product = Product::select_one(product_id, db_conn)
            .await?
            .filter(Product::is_listed)
            .ok_or(errors::OrderCreationError::ProductNonExistent(product_id))?;
        total_cost = total_cost
            .checked_add(
                u64::from(product.price())
                    .checked_mul(u64::from(count))
                    .ok_or(errors::OrderCreationError::CostTooLarge)?,
            )
            .ok_or(errors::OrderCreationError::CostTooLarge)?;
    }
    let order_insert = AppOrderInsert {
        amount_charged: i64::try_from(total_cost)
            .map_err(|_overflow| errors::OrderCreationError::CostTooLarge)?,
        order_placed: PrimitiveDateTime::new(current_time.date(), current_time.time()),
        user_id,
    };
    let order = order_insert.store(db_conn).await?;
    let order_id = order.id();
    for &(product_id, count) in &product_counts {
        let order_item_insert = OrderItemInsert::new(product_id, order_id, count);
        order_item_insert.store(db_conn).await?;
    }
    Ok(order)
}

/// TODO: add documentation
pub async fn search_orders(
    params: AppOrderSearchParameters,
    db_conn: &db::ConnectionPool,
) -> Result<Vec<AppOrder>, db::errors::DatabaseError> {
    AppOrder::search(params, db_conn).await
}

/// TODO: add documentation
pub async fn list_orders(
    db_conn: &db::ConnectionPool,
) -> Result<Vec<AppOrder>, db::errors::DatabaseError> {
    AppOrder::select_all(db_conn).await
}

/// TODO: add documentation
pub async fn delete_order(
    order_id: Uuid,
    db_conn: &db::ConnectionPool,
) -> Result<(), errors::OrderDeletionError> {
    match AppOrder::select_one(order_id, db_conn).await? {
        Some(order) => Ok(order.delete(db_conn).await?),
        None => Err(errors::OrderDeletionError::OrderNonExistent(order_id)),
    }
}

/// TODO: add documentation
pub async fn get_order(
    order_id: Uuid,
    db_conn: &db::ConnectionPool,
) -> Result<Option<AppOrder>, db::errors::DatabaseError> {
    AppOrder::select_one(order_id, db_conn).await
}

/// TODO: add documentation
pub async fn get_order_with_items(
    order_id: Uuid,
    db_conn: &db::ConnectionPool,
) -> Result<Option<AppOrderWithItems>, db::errors::DatabaseError> {
    let maybe_order = AppOrder::select_one(order_id, db_conn).await?;
    let Some(order) = maybe_order else {
        return Ok(None);
    };
    let order_items = OrderItem::select_all(order_id, db_conn).await?;
    Ok(Some(AppOrderWithItems {
        order,
        items: order_items
            .into_iter()
            .map(|item| (item.product_id(), item.count()))
            .collect(),
    }))
}

/// TODO: add documentation
pub async fn fulfil_order(
    order_id: Uuid,
    db_conn: &db::ConnectionPool,
) -> Result<(), errors::OrderFulfilmentError> {
    let mut order = AppOrder::select_one(order_id, db_conn)
        .await?
        .ok_or(errors::OrderFulfilmentError::OrderNonExistent(order_id))?;
    if order.status() != AppOrderStatus::Confirmed {
        return Err(errors::OrderFulfilmentError::OrderNotConfirmed(order_id));
    }
    order.set_status(AppOrderStatus::Fulfilled);
    order.update(db_conn).await?;
    Ok(())
}

/// Errors which can be returned by the orders service
pub mod errors {
    use crate::db::errors::DatabaseError;
    use thiserror::Error;
    use uuid::Uuid;

    #[derive(Error, Debug)]
    /// TODO: add documentation
    pub enum OrderConfirmationError {
        #[error(transparent)]
        /// TODO: add documentation
        DatabaseError(#[from] DatabaseError),
        #[error("Order does not exist")]
        /// TODO: add documentation
        OrderNonExistent(Uuid),
    }
    #[derive(Error, Debug)]
    /// TODO: add documentation
    pub enum OrderCreationError {
        #[error(transparent)]
        /// TODO: add documentation
        DatabaseError(#[from] DatabaseError),
        #[error("Product does not exist")]
        /// TODO: add documentation
        ProductNonExistent(Uuid),
        #[error("User does not exist")]
        /// TODO: add documentation
        UserNonExistent(Uuid),
        #[error("Total cost exceeds 64-bit max")]
        /// TODO: add documentation
        CostTooLarge,
    }

    #[derive(Error, Debug)]
    /// TODO: add documentation
    pub enum OrderFulfilmentError {
        #[error(transparent)]
        /// TODO: add documentation
        DatabaseError(#[from] DatabaseError),
        #[error("Order does not exist")]
        /// TODO: add documentation
        OrderNonExistent(Uuid),
        #[error("Order is not yet confirmed")]
        /// TODO: add documentation
        OrderNotConfirmed(Uuid),
    }

    #[derive(Error, Debug)]
    /// TODO: add documentation
    pub enum OrderDeletionError {
        #[error(transparent)]
        /// TODO: add documentation
        DatabaseError(#[from] DatabaseError),
        #[error("Order does not exist")]
        /// TODO: add documentation
        OrderNonExistent(Uuid),
    }
}
