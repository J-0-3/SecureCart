use serde::Serialize;
use time::{OffsetDateTime, PrimitiveDateTime};

use crate::db::{
    self,
    models::{
        apporder::{AppOrder, AppOrderInsert, AppOrderStatus},
        appuser::AppUser,
        order_item::{OrderItem, OrderItemInsert},
        product::Product,
    },
};

pub async fn confirm_order(
    order_id: u32,
    db_conn: &db::ConnectionPool,
) -> Result<(), errors::OrderConfirmationError> {
    let mut order = AppOrder::select_one(order_id, db_conn)
        .await?
        .ok_or(errors::OrderConfirmationError::OrderNonExistent)?;
    order.set_status(AppOrderStatus::Confirmed);
    order.update(db_conn).await?;
    Ok(())
}

#[derive(Serialize)]
pub struct AppOrderWithItems {
    pub order: AppOrder,
    pub items: Vec<(u32, u32)>, // id, count
}

pub async fn create_order(
    user_id: u32,
    product_counts: Vec<(u32, u32)>,
    db_conn: &db::ConnectionPool,
) -> Result<AppOrder, errors::OrderCreationError> {
    AppUser::select_one(user_id, db_conn)
        .await?
        .ok_or(errors::OrderCreationError::UserNonExistent)?;
    let current_time = OffsetDateTime::now_utc();
    let mut total_cost: u64 = 0;
    for &(product_id, count) in &product_counts {
        let product = Product::select_one(product_id, db_conn)
            .await?
            .ok_or(errors::OrderCreationError::ProductNonExistent)?;
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
        user_id: i64::from(user_id),
    };
    let order = order_insert.store(db_conn).await?;
    let order_id = order.id();
    for (product_id, count) in &product_counts {
        let order_item_insert = OrderItemInsert::new(*product_id, order_id, *count);
        order_item_insert.store(db_conn).await?;
    }
    Ok(order)
}

pub async fn search_orders(
    user_id: u32,
    db_conn: &db::ConnectionPool,
) -> Result<Vec<AppOrder>, db::errors::DatabaseError> {
    AppOrder::select_all_user(user_id, db_conn).await
}

pub async fn list_orders(
    db_conn: &db::ConnectionPool,
) -> Result<Vec<AppOrder>, db::errors::DatabaseError> {
    AppOrder::select_all(db_conn).await
}

pub async fn delete_order(
    order_id: u32,
    db_conn: &db::ConnectionPool,
) -> Result<(), errors::OrderDeletionError> {
    match AppOrder::select_one(order_id, db_conn).await? {
        Some(order) => Ok(order.delete(db_conn).await?),
        None => Err(errors::OrderDeletionError::OrderNonExistent),
    }
}

pub async fn get_order(
    order_id: u32,
    db_conn: &db::ConnectionPool,
) -> Result<Option<AppOrder>, db::errors::DatabaseError> {
    AppOrder::select_one(order_id, db_conn).await
}

pub async fn get_order_with_items(
    order_id: u32,
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

pub async fn fulfil_order(
    order_id: u32,
    db_conn: &db::ConnectionPool,
) -> Result<(), errors::OrderFulfilmentError> {
    let mut order = AppOrder::select_one(order_id, db_conn)
        .await?
        .ok_or(errors::OrderFulfilmentError::OrderNonExistent)?;
    order.set_status(AppOrderStatus::Fulfilled);
    order.update(db_conn).await?;
    Ok(())
}

pub mod errors {
    use crate::db::errors::DatabaseError;
    use thiserror::Error;

    #[derive(Error, Debug)]
    pub enum OrderConfirmationError {
        #[error(transparent)]
        DatabaseError(#[from] DatabaseError),
        #[error("Order does not exist")]
        OrderNonExistent,
    }
    #[derive(Error, Debug)]
    pub enum OrderCreationError {
        #[error(transparent)]
        DatabaseError(#[from] DatabaseError),
        #[error("Product does not exist")]
        ProductNonExistent,
        #[error("User does not exist")]
        UserNonExistent,
        #[error("Total cost exceeds 64-bit max")]
        CostTooLarge,
    }

    #[derive(Error, Debug)]
    pub enum OrderFulfilmentError {
        #[error(transparent)]
        DatabaseError(#[from] DatabaseError),
        #[error("Order does not exist")]
        OrderNonExistent,
    }

    #[derive(Error, Debug)]
    pub enum OrderDeletionError {
        #[error(transparent)]
        DatabaseError(#[from] DatabaseError),
        #[error("Order does not exist")]
        OrderNonExistent,
    }
}
