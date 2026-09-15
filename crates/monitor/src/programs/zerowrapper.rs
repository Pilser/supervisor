//! Zerowrapper (zw) monitoring config.

use super::registry::ProgramSpec;

/// Default spec for the zerowrapper binary (web UI).
pub fn zerowrapper_spec(binary: &str) -> ProgramSpec {
    ProgramSpec {
        name: "zerowrapper".to_string(),
        binary: binary.to_string(),
        args: vec![],
        restart_on_crash: true,
        // launch-zw.sh ends in `exec zw`: ps shows "zw", not the script.
        match_name: Some("zw".to_string()),
    }
}
