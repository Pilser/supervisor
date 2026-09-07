//! Dispatch of a hook event to every subscribed plugin.

use crate::hooks::HookDispatcher;
use crate::Result;
use core::HookPoint;

impl HookDispatcher {
    /// Deliver `event` to every plugin subscribed to `point`, in priority then
    /// insertion order. Each handler is invoked while the dispatcher lock is
    /// released. Errors are short-circuited back to the caller.
    pub fn dispatch(&self, point: HookPoint, event: &serde_json::Value) -> Result<()> {
        let handlers = self.snapshot(point);
        for entry in handlers {
            (entry.callback)(event)?;
        }
        Ok(())
    }
}