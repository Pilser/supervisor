//! `plugin_host` — the supervisor plugin host.
//!
//! Loads native `.so` supervisor plugins inside the VM via `libloading`,
//! watches the `plugins/` folder for changes, keeps a registry of loaded
//! plugins, dispatches `hook::HookPoint` events to subscribers and exposes an
//! inter-plugin API registry.
//!
//! This crate is deliberately a subset of the wrapper plugin host, scoped to
//! the supervisor `shared::Plugin` contract and `shared::HookPoint` variants.

pub mod api;
pub mod hooks;
pub mod lifecycle;
pub mod loader;
pub mod registry;
pub mod watcher;

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use libloading::Library;
use telemetry::{Level, Telemetry};

pub use shared::{
    HookPoint, Plugin, PluginApi, PluginContext, PluginInfo, PluginType, PluginVersion, Result,
    SupervisorError, SupervisorErrorKind,
};

use crate::hooks::HookDispatcher;
use crate::loader::{load_library, register_plugin, unregister_plugin};

/// A supervisor plugin that has been `dlopen`ed into the VM. Keeping the
/// `Library` alive pins the shared object in memory for the plugin's lifetime.
pub struct HandledPlugin {
    pub name: String,
    pub lib: Library,
    pub info: PluginInfo,
}

impl HandledPlugin {
    /// The plugin's registered name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// The statically-declared metadata resolved via `plugin_info`.
    pub fn info(&self) -> &PluginInfo {
        &self.info
    }
}

/// Top-level supervisor plugin host.
pub struct PluginHost {
    /// Directory scanned for `.so` supervisor plugins.
    pub folder: PathBuf,
    /// Name → loaded plugin map.
    pub registry: Mutex<HashMap<String, HandledPlugin>>,
    /// Central hook dispatcher, scoped to `shared::HookPoint`.
    pub dispatcher: HookDispatcher,
}

impl PluginHost {
    /// Build a host rooted at `folder` with a fresh dispatcher.
    pub fn new(folder: impl Into<PathBuf>) -> Self {
        Self {
            folder: folder.into(),
            registry: Mutex::new(HashMap::new()),
            dispatcher: HookDispatcher::new(),
        }
    }

    /// Build a host rooted at `folder`, using a caller-provided dispatcher.
    pub fn with_dispatcher(folder: impl Into<PathBuf>, dispatcher: HookDispatcher) -> Self {
        Self {
            folder: folder.into(),
            registry: Mutex::new(HashMap::new()),
            dispatcher,
        }
    }

    /// The plugin source directory being watched.
    pub fn folder(&self) -> &Path {
        &self.folder
    }

    /// The set of plugin files currently found under the folder.
    pub fn scan_files(&self) -> Result<Vec<PathBuf>> {
        watcher::scan::plugin_files(self.folder())
    }

    /// True if a plugin with `name` is currently registered.
    pub fn contains(&self, name: &str) -> bool {
        let reg = self.registry.lock().unwrap_or_else(|p| p.into_inner());
        reg.contains_key(name)
    }

    /// Clone the metadata of the registered plugin `name`, if present.
    pub fn lookup(&self, name: &str) -> Option<PluginInfo> {
        let reg = self.registry.lock().unwrap_or_else(|p| p.into_inner());
        reg.get(name).map(|p| p.info.clone())
    }

    /// Clone metadata for every registered plugin.
    pub fn list(&self) -> Vec<PluginInfo> {
        let reg = self.registry.lock().unwrap_or_else(|p| p.into_inner());
        reg.values().map(|p| p.info.clone()).collect()
    }

    /// Load every `.so` under the configured plugin folder. Emits an init
    /// event, then a `plugin.loaded` event per successful plugin and a
    /// `plugin.error` event for each one that fails to register.
    pub fn init(&self, telemetry: &Telemetry) -> Result<()> {
        std::fs::create_dir_all(self.folder()).map_err(SupervisorError::from)?;

        emit_plugin_event(
            telemetry,
            Level::Info,
            "plugin_host.init",
            "scanning plugin folder".to_string(),
            &self.folder.display().to_string(),
            None,
        );

        let files = self.scan_files()?;
        let mut loaded = 0usize;
        let mut failed = 0usize;
        for path in files {
            match self.load(&path, telemetry) {
                Ok(()) => loaded += 1,
                Err(e) => {
                    failed += 1;
                    emit_plugin_event(
                        telemetry,
                        Level::Error,
                        "plugin.error",
                        format!("failed to load `{}`: {e}", path.display()),
                        &path.display().to_string(),
                        Some(&e.to_string()),
                    );
                }
            }
        }

        emit_plugin_event(
            telemetry,
            if failed == 0 { Level::Info } else { Level::Warn },
            "plugin_host.init.complete",
            format!("loaded {loaded} plugins, {failed} failed"),
            &self.folder.display().to_string(),
            None,
        );
        Ok(())
    }

    /// Load a single `.so` plugin. If a plugin with the same name is already
    /// registered this is treated as a hot reload: the old library is
    /// unregistered and replaced.
    pub fn load(&self, path: &Path, telemetry: &Telemetry) -> Result<()> {
        let (lib, info) = load_library(path)?;
        let name = info.name.clone();

        // Evict any existing registration with the same name first.
        let evicting = self.evict(&name, telemetry);

        // Ask the plugin to register its runtime contract.
        let ctx = PluginContext::new(info.clone(), PluginApi::new(name.clone()));
        if let Err(e) = register_plugin(&lib, &ctx) {
            for &point in &info.hooks {
                let _ = self.dispatcher.unsubscribe(point, &name);
            }
            return Err(e);
        }

        // Subscribe the plugin to the hook points it declared.
        for &point in &info.hooks {
            let pname = name.clone();
            self.dispatcher
                .subscribe(point, pname, Arc::new(|_: &serde_json::Value| Ok(())))?;
        }

        {
            let mut reg = self.registry.lock().unwrap_or_else(|p| p.into_inner());
            reg.insert(
                name.clone(),
                HandledPlugin {
                    name: name.clone(),
                    lib,
                    info: info.clone(),
                },
            );
        }

        let (kind, hookpoint, verb) = if evicting {
            ("plugin.reloading".to_string(), HookPoint::OnPluginReloaded, "reloaded")
        } else {
            ("plugin.loaded".to_string(), HookPoint::OnPluginLoaded, "loaded")
        };
        emit_plugin_event(
            telemetry,
            Level::Info,
            &kind,
            format!("plugin `{name}` {verb}"),
            &name,
            None,
        );

        let event = serde_json::json!({ "plugin": name, "point": hookpoint.as_str() });
        let _ = self.dispatcher.dispatch(hookpoint, &event);
        Ok(())
    }

    /// Unregister and release the plugin named `name`.
    pub fn unload(&self, name: &str, telemetry: &Telemetry) -> Result<()> {
        let removed = {
            let mut reg = self.registry.lock().unwrap_or_else(|p| p.into_inner());
            reg.remove(name)
        };
        let Some(plugin) = removed else {
            return Err(SupervisorError::NotFound(format!("plugin `{name}`")));
        };

        let ctx = PluginContext::new(plugin.info.clone(), PluginApi::new(name.to_string()));
        let _ = unregister_plugin(&plugin.lib, &ctx);

        for &point in &plugin.info.hooks {
            let _ = self.dispatcher.unsubscribe(point, name);
        }

        drop(plugin);

        emit_plugin_event(
            telemetry,
            Level::Info,
            "plugin.unloaded",
            format!("plugin `{name}` unloaded"),
            name,
            None,
        );

        let event = serde_json::json!({ "plugin": name, "point": HookPoint::OnPluginUnloaded.as_str() });
        let _ = self.dispatcher.dispatch(HookPoint::OnPluginUnloaded, &event);
        Ok(())
    }

    /// Reload the plugin named `name` by re-reading its `.so` from disk and
    /// hot-swapping the registration.
    pub fn reload(&self, name: &str, telemetry: &Telemetry) -> Result<()> {
        let path = self.find_reload_path(name)?;
        if self.contains(name) {
            self.unload(name, telemetry)?;
        }
        self.load(&path, telemetry)
    }

    /// Locate the on-disk `.so` whose resolved name matches `name`.
    fn find_reload_path(&self, name: &str) -> Result<PathBuf> {
        for path in self.scan_files()? {
            if let Ok(lib) = loader::open(&path) {
                if let Ok(candidate) = loader::validate(&lib) {
                    if &candidate.name == name {
                        return Ok(path);
                    }
                }
            }
        }
        Err(SupervisorError::NotFound(format!("plugin `{name}` on disk")))
    }

    /// Remove any registration under `name`, best-effort unregistering the
    /// old library and detaching its hook subscriptions. Returns true if an
    /// entry was evicted.
    fn evict(&self, name: &str, _telemetry: &Telemetry) -> bool {
        let mut reg = self.registry.lock().unwrap_or_else(|p| p.into_inner());
        match reg.remove(name) {
            Some(old) => {
                let ctx = PluginContext::new(old.info.clone(), PluginApi::new(name.to_string()));
                let _ = unregister_plugin(&old.lib, &ctx);
                for &point in &old.info.hooks {
                    let _ = self.dispatcher.unsubscribe(point, name);
                }
                drop(old);
                true
            }
            None => false,
        }
    }
}

/// Emit a structured plugin event to telemetry.
fn emit_plugin_event(
    telemetry: &Telemetry,
    level: Level,
    kind: &str,
    message: String,
    subject: &str,
    error: Option<&str>,
) {
    let mut b = telemetry
        .builder()
        .level(level)
        .kind(kind)
        .message(message)
        .data("subject", subject)
        .tag("plugin_host");
    if let Some(err) = error {
        b = b.error(telemetry::ErrorPayload::new("plugin", err));
    }
    b.emit();
}