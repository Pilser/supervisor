//! Hook dispatch priorities. Lower values run first; ties are broken by
//! insertion order preserved in `HookEntry::order`.

use std::cmp::Ordering;

use crate::hooks::HookEntry;

/// Default priority applied to plugin subscriptions.
pub const DEFAULT_PRIORITY: i32 = 100;

/// Priority for subscriber runs before all normal plugins.
pub const HIGH_PRIORITY: i32 = 0;

/// Priority for subscriber runs after all normal plugins.
pub const LOW_PRIORITY: i32 = 200;

/// Total order over handler entries: lower priority first, then insertion
/// order.
pub fn order(a: &HookEntry, b: &HookEntry) -> Ordering {
    a.priority
        .cmp(&b.priority)
        .then_with(|| a.order.cmp(&b.order))
}