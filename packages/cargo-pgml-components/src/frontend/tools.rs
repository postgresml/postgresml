//! Tools required by us to build stuff.

use crate::util::{debug1, error, execute_command, info, print, unwrap_or_exit, warn};
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::process::{exit, Command};
use std::time::Duration;

// use notify::{Watcher, RecursiveMode, event::{EventKind, AccessKind}};
use notify_debouncer_full::{new_debouncer, notify::*, DebounceEventResult};

/// Required tools.
static TOOLS: &[&str] = &["sass", "rollup", "prettier"];
static ROLLUP_PLUGINS: &[&str] = &["@rollup/plugin-terser", "@rollup/plugin-node-resolve"];
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

    for plugin in ROLLUP_PLUGINS {
        if execute_with_nvm(Command::new("npm").arg("list").arg("-g").arg(plugin)).is_err() {
            warn(&format!("installing rollup plugin {}", plugin));
            unwrap_or_exit!(execute_with_nvm(
                Command::new("npm").arg("install").arg("-g").arg(plugin)
            ));
        }
    }

    if Path::new("package.json").exists() {
        info("installing dependencies from package.json");
        unwrap_or_exit!(execute_with_nvm(Command::new("npm").arg("install")));
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

    if let Err(err) = execute_with_nvm(Command::new("nvm").arg("--version")) {
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

pub fn debug() {
    let node = unwrap_or_exit!(execute_with_nvm(Command::new("which").arg("node")));
    println!("node: {}", node.trim());

    for tool in TOOLS {
        let output = unwrap_or_exit!(execute_with_nvm(Command::new("which").arg(tool)));
        println!("{}: {}", tool, output.trim());
    }
}

pub fn watch() {
    rebuild();

    let mut debouncer = unwrap_or_exit!(new_debouncer(
        Duration::from_secs(1),
        None,
        |result: DebounceEventResult| {
            match result {
                Ok(events) => {
                    let mut detected = true;
                    for event in &events {
                        for path in &event.event.paths {
                            let path = path.display().to_string();
                            if path.ends_with("modules.scss")
                                || path.contains("style.")
                                || path.ends_with(".pgml-bundle")
                                || path.ends_with("modules.js")
                                || path.contains("bundle.")
                                || path.ends_with(".rs")
                                || path.ends_with(".html")
                            {
                                detected = false;
                            }
                        }
                    }

                    if detected {
                        rebuild();
                    }
                }

                Err(e) => {
                    debug!("debouncer error: {:?}", e);
                }
            }
        }
    ));

    unwrap_or_exit!(debouncer
        .watcher()
        .watch(Path::new("src"), RecursiveMode::Recursive));
    unwrap_or_exit!(debouncer
        .watcher()
        .watch(Path::new("static"), RecursiveMode::Recursive));

    info("watching for changes");

    // sleep forever
    std::thread::sleep(std::time::Duration::MAX);
}

pub fn lint(check: bool) {
    let mut cmd = Command::new("prettier");
    if check {
        cmd.arg("--check");
    } else {
        cmd.arg("--write");
    }

    cmd.arg("src/**/*.js");

    print("linting...");

    let result = execute_with_nvm(&mut cmd);

    if let Err(err) = result {
        if check {
            error("diff detected");
        } else {
            error!("error");
        }

        error!("{}", err);
        exit(1);
    }

    info("ok");
}

fn rebuild() {
    print("changes detected, rebuilding...");
    match execute_command(Command::new("cargo").arg("pgml-components").arg("bundle")) {
        Ok(_) => info("ok"),
        Err(err) => {
            error("error");
            error!("{}", err);
        }
    }
}
