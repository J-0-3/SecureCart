use axum::{
    body::Body,
    extract::{FromRequest, State},
    http::{Request, StatusCode},
    response::{IntoResponse, Response},
    routing::post,
    Router,
};
use stripe::{Event, EventObject, EventType};
use uuid::Uuid;

use crate::{
    constants::stripe::STRIPE_WEBHOOK_SECRET,
    services::orders::{self, errors::OrderConfirmationError},
    state::AppState,
};

pub fn create_router() -> Router<AppState> {
    Router::new().route("/", post(stripe_webhook_event))
}

pub struct StripeEvent(Event);

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
                signature
                    .to_str()
                    .expect("Stripe webhook header value contained non-unicode characters"),
                &STRIPE_WEBHOOK_SECRET,
            )
            .map_err(|_err| {
                eprintln!("Invalid/Unauthenticated stripe webhook event");
                StatusCode::BAD_REQUEST.into_response()
            })?,
        ))
    }
}

pub async fn stripe_webhook_event(
    State(state): State<AppState>,
    StripeEvent(event): StripeEvent,
) -> Result<(), StatusCode> {
    #[expect(
        clippy::wildcard_enum_match_arm,
        reason = "There are over 400 possible stripe webhook events. I refuse to list them all."
    )]
    match event.type_ {
        EventType::PaymentIntentSucceeded => {
            if let EventObject::PaymentIntent(data) = event.data.object {
                let order_id: Uuid = data.metadata.get("order_id").ok_or_else(|| {
                    eprintln!("Stripe webhook paymentintent.succeeded did not contain order_id metadata");
                    StatusCode::BAD_REQUEST
                })?
                .parse().map_err(|_parse| {
                    eprintln!("Stripe webhook paymentintent order_id not an integer");
                    StatusCode::UNPROCESSABLE_ENTITY
                })?;
                orders::confirm_order(order_id, &state.db)
                    .await
                    .map_err(|error| match error {
                        OrderConfirmationError::DatabaseError(err) => {
                            eprintln!("Error raised by database while confirming order: {err}");
                            StatusCode::INTERNAL_SERVER_ERROR
                        }
                        OrderConfirmationError::OrderNonExistent(order_id) => {
                            eprintln!(
                                "Stripe attempted to confirm order {order_id}, which does not exist."
                            );
                            StatusCode::NOT_FOUND
                        }
                    })?;
            }
            Ok(())
        }
        _ => Ok(()),
    }
}
