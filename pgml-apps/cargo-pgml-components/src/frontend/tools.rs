//! Tools required by us to build stuff.

use crate::util::{debug1, error, execute_command, unwrap_or_exit, warn};
use std::fs::File;
use std::io::Write;
use std::process::{exit, Command};

/// Required tools.
static TOOLS: &[&str] = &["sass", "rollup"];
static NVM_EXEC: &'static str = "/tmp/pgml-components-nvm.sh";
static NVM_SOURCE: &'static str = "https://raw.githubusercontent.com/nvm-sh/nvm/v0.39.5/install.sh";
static NVM_SOURCE_DOWNLOADED: &'static str = "/tmp/pgml-components-nvm-source.sh";

/// Install any missing tools.
pub fn install() {
    install_nvm_entrypoint();
    debug!("installed node entrypoint");
    install_node();
    debug!("installed node");

    for tool in TOOLS {
        match execute_with_nvm(Command::new(tool).arg("--version")) {
            Ok(_) => (),
            Err(err) => {
                debug1!(err);
                warn(&format!("installing {}", tool));
                unwrap_or_exit!(execute_with_nvm(
                    Command::new("npm").arg("install").arg("-g").arg(tool)
                ));
            }
        }
    }
}

/// Execute a command making sure that nvm is available.
pub fn execute_with_nvm(command: &mut Command) -> std::io::Result<String> {
    let mut cmd = Command::new(NVM_EXEC);
    cmd.arg(command.get_program());
    for arg in command.get_args() {
        cmd.arg(arg);
    }
    execute_command(&mut cmd)
}

/// Install the nvm entrypoint we provide into /tmp
fn install_nvm_entrypoint() {
    let mut file = unwrap_or_exit!(File::create(NVM_EXEC));
    unwrap_or_exit!(writeln!(&mut file, "{}", include_str!("nvm.sh")));
    drop(file);

    unwrap_or_exit!(execute_command(
        Command::new("chmod").arg("+x").arg(NVM_EXEC)
    ));
}

/// Install node using nvm
fn install_node() {
    debug!("installing node");
    // Node is already installed.
    if let Ok(_) = execute_with_nvm(Command::new("node").arg("--version")) {
        debug!("node is available");
        return;
    }

    warn("installing node using nvm");

    debug!("node is not available");

    if let Err(err) = execute_command(Command::new("nvm").arg("--version")) {
        debug!("nvm is not available");
        debug1!(err);
        // Install Node Version Manager.
        if let Err(err) = execute_command(
            Command::new("curl")
                .arg("-Ls")
                .arg(NVM_SOURCE)
                .arg("-o")
                .arg(NVM_SOURCE_DOWNLOADED),
        ) {
            debug!("curl is not available");
            error("couldn't not download nvm from Github, please do so manually before proceeding");
            debug1!(err);
            exit(1);
        } else {
            if let Err(err) = execute_command(Command::new("bash").arg(NVM_SOURCE_DOWNLOADED)) {
                error("couldn't install nvm, please do so manually before proceeding");
                debug1!(err);
                exit(1);
            } else {
                warn("installed nvm");
            }
        }
    }

    if let Err(err) = execute_with_nvm(Command::new("nvm").arg("install").arg("stable")) {
        error("couldn't install Node, please do so manually before proceeding");
        debug1!(err);
        exit(1);
    } else {
        warn("installed node")
    }
}
