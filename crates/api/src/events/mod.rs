//! Plugin event bus: emit and subscribe.

pub mod emit;
pub mod subscribe;

pub use emit::emit_api_event;
pub use subscribe::EventSubscription;
