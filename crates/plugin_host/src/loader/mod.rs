//! Plugin loader — opens `.so` supervisor plugins and resolves their FFI
//! entry points.

pub mod load;
pub mod symbols;
pub mod unload;
pub mod validate;

pub use load::{load_library, open, register_plugin};
pub use unload::unregister_plugin;
pub use validate::validate;