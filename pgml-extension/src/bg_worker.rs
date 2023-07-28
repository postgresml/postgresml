//! Setup and sendd API calls to a Postgres BG worker
use std::{io, os::unix::net::UnixStream, path::PathBuf, time::Duration};

use bgworker_message::{Reply, Request};
use once_cell::sync::OnceCell;
use pgrx::{bgworkers::BackgroundWorkerBuilder, IntoDatum};

use crate::config::get_config;

const RESTART_DURATION: Duration = Duration::from_secs(1);
static NAME: &str = "pgml_bgworker";
static BG_WORKER_SOCKET_PATH: OnceCell<PathBuf> = OnceCell::new();
static CONFIG_LIBRARY_KEY: &str = "pgml.bgworker_library";
static CONFIG_FUNCTION_KEY: &str = "pgml.bgworker_entry_fn";
static CONFIG_SOCKET_PATH_KEY: &str = "pgml.bgworker_socket_path";

macro_rules! get_config_or_return {
    ($name:expr) => {
        match get_config($name) {
            Some(v) => v,
            None => return,
        }
    };
}

/// Setup the bg_worker using values in `postgresql.conf`.
pub fn setup() {
    let library = get_config_or_return!(CONFIG_LIBRARY_KEY);
    let function = get_config_or_return!(CONFIG_FUNCTION_KEY);
    let socket_path = get_config_or_return!(CONFIG_SOCKET_PATH_KEY);

    load_bg_worker(&library, &function, &socket_path);
}

/// Returns whether a bg_worker is being used.
pub fn enabled() -> bool {
    BG_WORKER_SOCKET_PATH.get().is_some()
}

fn load_bg_worker(library: &str, function: &str, socket_path: &str) {
    // UNWRAP: This should only be called once
    BG_WORKER_SOCKET_PATH
        .set(PathBuf::from(socket_path))
        .unwrap();

    BackgroundWorkerBuilder::new(NAME)
        .enable_spi_access()
        .set_function(function)
        .enable_shmem_access(None) // shmem required
        .set_library(library)
        .set_restart_time(Some(RESTART_DURATION))
        .set_argument(socket_path.into_datum())
        .load();
}

/// Send a `Request` to the bg_worker and wait for a `Reply`.
pub fn send_request(request: Request) -> io::Result<Reply> {
    /* Connect to socket */
    // UNWRAP: Should always be set by this time
    let path = BG_WORKER_SOCKET_PATH.get().unwrap();
    let mut stream = UnixStream::connect(path)?;

    /* Send request */
    request.send(&mut stream)?;

    /* Read reply */
    Reply::recv(&mut stream)
}
