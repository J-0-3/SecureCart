//! API routes within the application. Mainly exposes sub-routers which should
//! be nested with the main Axum router.
pub mod auth;
pub mod checkout;
pub mod orders;
pub mod products;
pub mod registration;
pub mod users;
pub mod webhook;
