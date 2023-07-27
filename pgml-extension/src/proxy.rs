//! Proxy API calls to a Postgres BG worker
use std::{
    io::{self, Read, Write},
    net::{SocketAddr, TcpStream},
    time::Duration,
};

use once_cell::sync::OnceCell;
use pgrx::{bgworkers::BackgroundWorkerBuilder, error, IntoDatum};
use proxy_message::{Reply, Request, REPLY_SIZE};

use crate::config::get_config;

const RESTART_DURATION: Duration = Duration::from_secs(1);
static NAME: &str = "pgml_proxy";
static PROXY_ADDR: OnceCell<SocketAddr> = OnceCell::new();
static CONFIG_LIBRARY_KEY: &str = "pgml.proxy.library";
static CONFIG_FUNCTION_KEY: &str = "pgml.proxy.entry_fn";
static CONFIG_SOCKET_ADDR_KEY: &str = "pgml.proxy.socket_addr";

macro_rules! get_config_or_return {
    ($name:expr) => {
        match get_config($name) {
            Some(v) => v,
            None => return,
        }
    };
}

/// Setup the proxy bg_worker using values in `postgresql.conf`.
pub fn setup() {
    let library = get_config_or_return!(CONFIG_LIBRARY_KEY);
    let function = get_config_or_return!(CONFIG_FUNCTION_KEY);
    let socket_addr = get_config_or_return!(CONFIG_SOCKET_ADDR_KEY);

    load_bg_worker(&library, &function, &socket_addr);
}

/// Returns whether a proxy is being used.
pub fn enabled() -> bool {
    PROXY_ADDR.get().is_some()
}

fn load_bg_worker(library: &str, function: &str, socket_addr: &str) {
    match socket_addr.parse::<SocketAddr>() {
        Ok(addr) => {
            // UNWRAP: This is the only place it is set and should only be called once
            PROXY_ADDR.set(addr).unwrap();
        }
        Err(e) => error!("could not parse proxy address: {e}"),
    };

    BackgroundWorkerBuilder::new(NAME)
        .set_function(function)
        .enable_shmem_access(None)
        .set_library(library)
        .set_restart_time(Some(RESTART_DURATION))
        .set_argument(socket_addr.into_datum())
        .load();
}

/// Send a `Request` to the proxy and wait for a `Reply`.
pub fn send_request(request: Request) -> io::Result<Reply> {
    let addr = PROXY_ADDR.get().unwrap();
    let mut stream = TcpStream::connect(addr)?;

    let bytes = request.encode().unwrap();
    let _ = stream.write(&bytes)?;

    let mut bytes = [0u8; REPLY_SIZE];
    let _ = stream.read(&mut bytes)?;

    Reply::decode(&bytes).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
}
