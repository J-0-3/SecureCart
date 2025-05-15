use axum::{
    extract::State,
    http::StatusCode,
    middleware::from_fn_with_state,
    routing::{get, post},
    Extension, Json, Router,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    middleware::session::session_middleware,
    services::{checkout, orders, sessions::CustomerSession},
    state::AppState,
    utils::httperror::HttpError,
};

#[cfg(feature = "stripe")]
use crate::constants::stripe::STRIPE_PUBLISHABLE_KEY;

pub fn create_router(state: &AppState) -> Router<AppState> {
    let customer = Router::new()
        .route("/", post(do_checkout))
        .layer(from_fn_with_state(
            state.clone(),
            session_middleware::<CustomerSession>,
        ));
    let unauthenticated = Router::new().route("/", get(get_status));
    customer.merge(unauthenticated)
}

#[derive(Serialize)]
struct CheckoutStatusResponse {
    stripe_enabled: bool,
    stripe_publishable_key: Option<String>,
}

async fn get_status() -> Json<CheckoutStatusResponse> {
    Json(CheckoutStatusResponse {
        stripe_enabled: cfg!(feature = "stripe"),
        #[cfg(feature = "stripe")]
        stripe_publishable_key: Some(STRIPE_PUBLISHABLE_KEY.clone()),
        #[cfg(not(feature = "stripe"))]
        stripe_publishable_key: None,
    })
}

#[derive(Deserialize)]
struct CheckoutRequestBody {
    order_id: Uuid,
}

#[derive(Serialize)]
struct CheckoutResponsePaymentInfo {
    publishable_key: String,
    client_secret: String,
}
#[derive(Serialize)]
struct CheckoutRequestResponse {
    payment_required: bool,
    payment_info: Option<CheckoutResponsePaymentInfo>,
}

async fn do_checkout(
    State(state): State<AppState>,
    Extension(session): Extension<CustomerSession>,
    Json(body): Json<CheckoutRequestBody>,
) -> Result<Json<CheckoutRequestResponse>, HttpError> {
    let user_id = session.user_id();
    let checkout_token = checkout::CheckoutToken::create(user_id, body.order_id, &state.db).await?;
    if cfg!(not(feature = "stripe")) {
        println!(
            "Stripe is disabled, unconditionally confirming order {} without payment.",
            body.order_id
        );
        orders::confirm_order(body.order_id, &state.db).await?;
        Ok(Json(CheckoutRequestResponse {
            payment_required: false,
            payment_info: None,
        }))
    } else {
        let client_secret = checkout_token
            .client_secret()
            .expect("Somehow client secret was None with stripe feature set. Seriously broken.");
        Ok(Json(CheckoutRequestResponse {
            payment_required: true,
            payment_info: Some(CheckoutResponsePaymentInfo {
                client_secret,
                #[cfg(feature = "stripe")]
                publishable_key: STRIPE_PUBLISHABLE_KEY.clone(),
                // just to appease the compiler, impossible for the feature to be both on and off
                #[cfg(not(feature = "stripe"))]
                publishable_key: String::from("BAD"), // this will never ever happen
            }),
        }))
    }
}

impl From<orders::errors::OrderConfirmationError> for HttpError {
    fn from(error: orders::errors::OrderConfirmationError) -> Self {
        match error {
            orders::errors::OrderConfirmationError::DatabaseError(err) => err.into(),
            orders::errors::OrderConfirmationError::OrderNonExistent(order_id) => {
                eprintln!("Attempted to confirm order {order_id}, which does not exist");
                Self::new(
                    StatusCode::NOT_FOUND,
                    Some(format!("Order {order_id} not found.")),
                )
            }
        }
    }
}
impl From<checkout::errors::CheckoutTokenCreateError> for HttpError {
    fn from(error: checkout::errors::CheckoutTokenCreateError) -> Self {
        match error {
            checkout::errors::CheckoutTokenCreateError::DatabaseError(err) => err.into(),
            checkout::errors::CheckoutTokenCreateError::Unauthorized { user_id, order_id } => {
                eprintln!(
                    "User {user_id} made an unauthorized attempt to checkout for order {order_id}"
                );
                Self::from(StatusCode::FORBIDDEN)
            }
            checkout::errors::CheckoutTokenCreateError::OrderNonExistent { user_id, order_id } => {
                eprintln!("User {user_id} attempted to checkout for non-existent order {order_id}");
                Self::from(StatusCode::FORBIDDEN) // not 404 to prevent enumerating valid orders
            }
            #[cfg(feature = "stripe")]
            checkout::errors::CheckoutTokenCreateError::StripeError(err) => {
                eprintln!("Stripe error when initialising checkout: {err}");
                Self::from(StatusCode::INTERNAL_SERVER_ERROR) // don't want to accidentally leak ANYTHING about stripe
            }
        }
    }
}
