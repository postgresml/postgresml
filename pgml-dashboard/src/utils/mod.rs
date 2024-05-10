pub mod config;
pub mod cookies;
pub mod datadog;
pub mod markdown;
pub mod tabs;
pub mod time;
pub mod urls;

use rand::{distributions::Alphanumeric, Rng};

/// Generate a random string of any length.
pub fn random_string(len: usize) -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(len)
        .map(char::from)
        .collect()
}
