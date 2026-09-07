//! `deployment` — receives plugins, configs, binaries and commands from the
//! wrapper based on the tenant's category.
//!
//! Listens for deployment pushes, verifies checksums, installs each artifact
//! kind, handles category changes with snapshot/rollback, and reports status
//! back to the wrapper.

#![forbid(unsafe_code)]

pub mod artifacts;
pub mod category;
pub mod install;
pub mod receiver;
pub mod report;
pub mod rollback;

pub use artifacts::{parse_manifest, DeploymentArtifact, DeploymentManifest};
pub use rollback::{cleanup_snapshot, revert_snapshot, take_snapshot, Snapshot};
