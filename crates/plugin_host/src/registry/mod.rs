//! In-memory registry of loaded supervisor plugins, backed by the `HashMap`
//! owned by `PluginHost`. These helpers operate on the shared registry so the
//! host does not need to duplicate storage logic.

pub mod dependency;
pub mod lookup;
pub mod store;

pub use dependency::resolve_load_order;
pub use lookup::{by_name, contained, names};
pub use store::{insert_unless_present, register_file_under_name};