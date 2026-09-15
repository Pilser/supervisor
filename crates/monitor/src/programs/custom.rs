//! Register custom programs to monitor (plugin hook surface).

use super::registry::ProgramSpec;

/// Build a spec for any additional supervised program.
pub fn custom_spec(name: &str, binary: &str, args: Vec<String>) -> ProgramSpec {
    ProgramSpec {
        name: name.to_string(),
        binary: binary.to_string(),
        args,
        restart_on_crash: true,
        match_name: None,
    }
}
