//! Tools required by us to build stuff.

use crate::util::{error, execute_command, unwrap_or_exit, warn};
use std::process::{exit, Command};

/// Required tools.
static TOOLS: &[&str] = &["sass", "rollup"];

/// Install any missing tools.
pub fn install() {
    if let Err(err) = execute_command(Command::new("node").arg("--version")) {
        error("Node is not installed. Install it with nvm or your system package manager.");
        debug!("{}", err);
        exit(1);
    }

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
