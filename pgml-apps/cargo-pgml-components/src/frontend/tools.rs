//! Tools required by us to build stuff.

use crate::util::{execute_command, info, unwrap_or_exit, warn};
use std::process::Command;

/// Required tools.
static TOOLS: &[&str] = &["sass", "rollup"];

/// Install any missing tools.
pub fn install() {
    for tool in TOOLS {
        match execute_command(Command::new(tool).arg("--version")) {
            Ok(_) => (),
            Err(err) => {
                warn(&format!("installing {}", tool));
                unwrap_or_exit!(execute_command(
                    Command::new("npm").arg("install").arg("-g").arg(tool)
                ));
            }
        }
    }
}
