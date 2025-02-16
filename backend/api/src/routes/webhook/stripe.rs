use axum::{
    body::Body,
    extract::{FromRequest, State},
    http::{Request, StatusCode},
    response::{IntoResponse, Response},
    routing::post,
    Router,
};
use stripe::{Event, EventObject, EventType, PaymentIntent};

use crate::{
    constants::stripe::STRIPE_SECRET_KEY,
    services::orders::{self, errors::OrderConfirmationError},
    state::AppState,
};

pub fn create_router(state: &AppState) -> Router<AppState> {
    Router::new().route("/", post(stripe_webhook_event))
}

struct StripeEvent(Event);

impl FromRequest<AppState> for StripeEvent
where
    String: FromRequest<AppState>,
{
    type Rejection = Response;

    async fn from_request(req: Request<Body>, state: &AppState) -> Result<Self, Self::Rejection> {
        let signature = if let Some(sig) = req.headers().get("stripe-signature") {
            sig.to_owned()
        } else {
            return Err(StatusCode::BAD_REQUEST.into_response());
        };

        let payload = String::from_request(req, state)
            .await
            .map_err(IntoResponse::into_response)?;

        Ok(Self(
            stripe::Webhook::construct_event(
                &payload,
                signature.to_str().unwrap(),
                &*STRIPE_SECRET_KEY,
            )
            .map_err(|_| StatusCode::BAD_REQUEST.into_response())?,
        ))
    }
}

pub async fn stripe_webhook_event(State(state): State<AppState>, StripeEvent(event): StripeEvent) {
    match event.type_ {
        EventType::PaymentIntentSucceeded => {
            if let EventObject::PaymentIntent(data) = event.data.object {
                let order_id: u32 = data.metadata["order_id"]
                    .parse()
                    .expect("Stripe webhook paymentintent order_id not an integer");
                orders::confirm_order(order_id, &state.db)
                    .await
                    .map_err(|err| match err {
                        OrderConfirmationError::DatabaseError(err) => {
                            eprintln!("Error raised by database while confirming order: {err}")
                        }
                        OrderConfirmationError::OrderNonExistent => {
                            eprintln!(
                                "Attempted to confirm order {order_id}, which does not exist."
                            )
                        }
                    });
            }
        }
        _ => {}
    }
}
