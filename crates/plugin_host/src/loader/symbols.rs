//! Unsafe FFI symbol resolution and invocation.
//!
//! This is the plugin loader boundary: it reaches across the C ABI into the
//! `.so` plugin (which is trusted supervisor code shipped into the VM). The
//! three required entry points are `plugin_info`, `register` and `unregister`.
//!
//! Symbols are resolved lazily and invoked at call time so the registry never
//! stores dangling `Symbol<'static>` references — the owning `Library` is kept
//! alive by `HandledPlugin`.
//!
//! Panics that escape the plugin across the FFI boundary are caught and mapped
//! into `SupervisorError::Plugin`, as are malformed return payloads.

use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::panic::{self, AssertUnwindSafe};

use libloading::{Library, Symbol};

use crate::{PluginContext, PluginInfo, SupervisorError};

/// `plugin_info() -> *const c_char` returns a NUL-terminated JSON `PluginInfo`.
pub type PluginInfoFn = unsafe extern "C" fn() -> *const c_char;

/// `register(ctx_json) -> *const c_char` / `unregister(ctx_json) -> *const c_char`.
/// Returns NULL on success or a NUL-terminated JSON `{"ok":false,"error":"..."}`.
pub type PluginHookFn = unsafe extern "C" fn(*const c_char) -> *const c_char;

fn resolve<'l, T>(lib: &'l Library, name: &[u8]) -> crate::Result<Symbol<'l, T>> {
    unsafe { lib.get::<T>(name) }.map_err(|e| {
        SupervisorError::Plugin(format!(
            "plugin is missing required symbol `{}`: {e}",
            String::from_utf8_lossy(name).trim_end_matches('\0')
        ))
    })
}

/// Resolve the `plugin_info` entry point.
pub fn symbol_info(lib: &Library) -> crate::Result<Symbol<'_, PluginInfoFn>> {
    resolve(lib, b"plugin_info\0")
}

/// Resolve the `register` entry point.
pub fn symbol_register(lib: &Library) -> crate::Result<Symbol<'_, PluginHookFn>> {
    resolve(lib, b"register\0")
}

/// Resolve the `unregister` entry point.
pub fn symbol_unregister(lib: &Library) -> crate::Result<Symbol<'_, PluginHookFn>> {
    resolve(lib, b"unregister\0")
}

/// Call `plugin_info` and deserialize the returned JSON into `PluginInfo`.
///
/// # Safety
/// The target library must actually export a compliant `plugin_info` symbol.
pub unsafe fn call_info(lib: &Library) -> crate::Result<PluginInfo> {
    let sym = symbol_info(lib)?;
    let result = panic::catch_unwind(AssertUnwindSafe(|| {
        unsafe {
            let ptr = sym();
            if ptr.is_null() {
                return Err(SupervisorError::Plugin(
                    "`plugin_info` returned a null pointer".into(),
                ));
            }
            let raw = CStr::from_ptr(ptr);
            let json = raw.to_str().map_err(|e| {
                SupervisorError::Plugin(format!("`plugin_info` returned non-UTF8 data: {e}"))
            })?;
            serde_json::from_str::<PluginInfo>(json).map_err(SupervisorError::from)
        }
    }));
    match result {
        Ok(r) => r,
        Err(_) => Err(SupervisorError::Plugin(
            "`plugin_info` panicked across the FFI boundary".into(),
        )),
    }
}

/// Invoke a `register`/`unregister`-style entry point with a serialized
/// `PluginContext`.
///
/// # Safety
/// The target library must export a compliant hook entry point.
pub unsafe fn call_register(lib: &Library, ctx: &PluginContext) -> crate::Result<()> {
    let sym = symbol_register(lib)?;
    invoke(&sym, ctx)
}

/// # Safety
/// The target library must export a compliant `unregister` entry point.
pub unsafe fn call_unregister(lib: &Library, ctx: &PluginContext) -> crate::Result<()> {
    let sym = symbol_unregister(lib)?;
    invoke(&sym, ctx)
}

fn invoke(sym: &Symbol<'_, PluginHookFn>, ctx: &PluginContext) -> crate::Result<()> {
    // `PluginContext` itself is not `Serialize` (its `PluginApi` extensions
    // map is runtime state), so project the FFI-stable fields into JSON.
    let payload = serde_json::json!({
        "plugin": ctx.info.name,
        "info": ctx.info,
        "plugin_id": ctx.api.plugin_id,
        "config": ctx.config,
    });
    let json = serde_json::to_string(&payload).map_err(SupervisorError::from)?;
    let c_json = CString::new(json)
        .map_err(|_| SupervisorError::Plugin("context contains an interior NUL byte".into()))?;

    let result = panic::catch_unwind(AssertUnwindSafe(|| {
        let ptr = unsafe { sym(c_json.as_ptr()) };
        if ptr.is_null() {
            return Ok(());
        }
        let raw = unsafe { CStr::from_ptr(ptr) };
        let payload = raw.to_string_lossy();
        let value: serde_json::Value = serde_json::from_str(&payload).map_err(SupervisorError::from)?;
        if value
            .get("ok")
            .and_then(|o| o.as_bool())
            .unwrap_or(false)
        {
            return Ok(());
        }
        let msg = value
            .get("error")
            .and_then(|e| e.as_str())
            .unwrap_or("plugin returned a failure");
        Err(SupervisorError::Plugin(msg.to_string()))
    }));

    match result {
        Ok(r) => r,
        Err(_) => Err(SupervisorError::Plugin(
            "plugin entry point panicked across the FFI boundary".into(),
        )),
    }
}