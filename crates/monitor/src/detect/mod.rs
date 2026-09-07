//! Failure detectors: crash, freeze, leak, zombie.

pub mod crash;
pub mod freeze;
pub mod leak;
pub mod zombie;

pub use crash::detect_crash;
pub use freeze::FreezeDetector;
pub use leak::LeakDetector;
pub use zombie::detect_zombie;
