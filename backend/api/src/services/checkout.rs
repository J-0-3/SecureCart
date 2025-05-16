//! Logic for handling checkouts, with or without Stripe integrated.
#[cfg(feature = "stripe")]
use crate::constants::stripe::STRIPE_SECRET_KEY;
use crate::db::{self, models::apporder::AppOrder};
#[cfg(feature = "stripe")]
use stripe;
use uuid::Uuid;

#[cfg(feature = "stripe")]
/// A live checkout token containing a stripe PaymentIntent.
pub struct CheckoutToken(stripe::PaymentIntent);

#[cfg(not(feature = "stripe"))]
/// A mock checkout token not including a Stripe `PaymentIntent`.
pub struct CheckoutToken;

impl CheckoutToken {
    #[cfg(feature = "stripe")]
    pub async fn create(
        user_id: Uuid,
        order_id: Uuid,
        db_conn: &db::ConnectionPool,
    ) -> Result<Self, errors::CheckoutTokenCreateError> {
        use core::iter;
        let order = AppOrder::select_one(order_id, db_conn)
            .await?
            .ok_or(errors::CheckoutTokenCreateError::OrderNonExistent { user_id, order_id })?;
        if order.user_id() != user_id {
            return Err(errors::CheckoutTokenCreateError::Unauthorized { user_id, order_id });
        }
        let stripe_client = stripe::Client::new(&*STRIPE_SECRET_KEY);
        let mut create_intent =
            stripe::CreatePaymentIntent::new(order.amount_charged, stripe::Currency::GBP);
        create_intent.payment_method_types = Some(vec!["card".to_owned()]);
        create_intent.metadata =
            Some(iter::once(("order_id".to_owned(), order_id.to_string())).collect());
        Ok(Self(
            stripe::PaymentIntent::create(&stripe_client, create_intent).await?,
        ))
    }
    #[cfg(feature = "stripe")]
    /// Returns the Stripe payment intent client secret. Always returns Some,
    /// the Option is for handling builds where stripe is disabled.
    #[expect(
        clippy::unnecessary_wraps,
        reason = "Option is used here for conditional compilation"
    )]
    #[expect(
        clippy::unwrap_in_result,
        reason = "This function should not fail, and will always return Some"
    )]
    pub fn client_secret(&self) -> Option<String> {
        Some(self.0.client_secret.clone().expect(
            "Payment intent does not contain a client secret. Something has gone seriously wrong.",
        ))
    }
    #[cfg(not(feature = "stripe"))]
    pub async fn create(
        user_id: Uuid,
        order_id: Uuid,
        db_conn: &db::ConnectionPool,
    ) -> Result<Self, errors::CheckoutTokenCreateError> {
        let order = AppOrder::select_one(order_id, db_conn)
            .await?
            .ok_or(errors::CheckoutTokenCreateError::OrderNonExistent { user_id, order_id })?;
        if order.user_id() == user_id {
            Ok(Self)
        } else {
            Err(errors::CheckoutTokenCreateError::Unauthorized { user_id, order_id })
        }
    }
    #[cfg(not(feature = "stripe"))]
    #[expect(
        clippy::unused_self,
        reason = "This is a mock method, must match the real signature"
    )]
    /// Always returns None, would return Some if stripe were enabled.
    pub const fn client_secret(&self) -> Option<String> {
        None
    }
}

/// TODO: add documentation
pub mod errors {
    use crate::db::errors::DatabaseError;
    use thiserror::Error;
    use uuid::Uuid;

    #[derive(Debug, Error)]
    /// TODO: add documentation
    pub enum CheckoutTokenCreateError {
        #[error(transparent)]
        /// TODO: add documentation
        DatabaseError(#[from] DatabaseError),
        #[error("Attempted to create a checkout token for a non-existent order ID")]
        /// TODO: add documentation
        OrderNonExistent {
            /// TODO: add documentation
            user_id: Uuid,
            /// TODO: add documentation
            order_id: Uuid,
        },
        #[error("The user ID does not match the owned of the order ID supplied")]
        /// TODO: add documentation
        Unauthorized {
            /// TODO: add documentation
            user_id: Uuid,
            /// TODO: add documentation
            order_id: Uuid,
        },
        #[cfg(feature = "stripe")]
        #[error(transparent)]
        StripeError(#[from] stripe::StripeError),
    }
}
