//! Programs under supervision: registry plus agent/memory/custom specs.

pub mod agent;
pub mod custom;
pub mod memory;
pub mod registry;

pub use agent::agent_spec;
pub use custom::custom_spec;
pub use memory::memory_spec;
pub use registry::{ProgramRegistry, ProgramSpec};
