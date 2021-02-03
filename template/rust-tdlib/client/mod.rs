//! Module contains structs and traits, required for proper interaction with Telegram server.
#[doc(hidden)]
mod observer;

/// TDlib API methods.
pub mod client;
#[allow(clippy::module_inception)]
/// Handlers for all incoming data
pub mod worker;
pub mod errors;

pub use worker::{AuthStateHandler, WorkerBuilder, ConsoleAuthStateHandler};
pub use client::Client;
