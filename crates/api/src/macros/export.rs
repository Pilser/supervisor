//! Plugin export macro.

/// Declare the plugin entry point name. Stub for the codegen macro.
#[macro_export]
macro_rules! export_plugin {
    ($name:expr) => {
        #[doc = "Exported plugin entry point."]
        pub const EXPORTED_PLUGIN_NAME: &str = $name;
    };
}
