//! The `Plugin` trait every supervisor plugin conforms to.

use super::{PluginContext, PluginInfo};
use crate::Result;

pub trait Plugin: Send + Sync {
    fn info(&self) -> PluginInfo;

    fn register(&mut self, ctx: PluginContext) -> Result<()> {
        let _ = ctx;
        Ok(())
    }

    fn unregister(&mut self, ctx: PluginContext) -> Result<()> {
        let _ = ctx;
        Ok(())
    }
}