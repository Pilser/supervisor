//! Category handling: detect, request, change.

pub mod change;
pub mod detect;
pub mod request;

pub use change::handle_category_change;
pub use detect::detect_category;
pub use request::request_deployment;
