//! Programs under supervision: registry plus engram/zerowrapper/custom specs.

pub mod custom;
pub mod engram;
pub mod registry;
pub mod zerowrapper;

pub use custom::custom_spec;
pub use engram::engram_spec;
pub use registry::{ProgramRegistry, ProgramSpec};
pub use zerowrapper::zerowrapper_spec;
