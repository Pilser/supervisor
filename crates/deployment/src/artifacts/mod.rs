//! Artifact handlers per type: plugin, agent plugin, config, binary,
//! command, manifest.

pub mod agent_plugin;
pub mod binary;
pub mod command;
pub mod config;
pub mod manifest;
pub mod plugin;

pub use core::{DeploymentArtifact, DeploymentManifest};
pub use manifest::parse_manifest;
