//! Deployment reporting: status and history.

pub mod history;
pub mod status;

pub use history::DeploymentHistory;
pub use status::{deployment_status, report_to_wrapper};
