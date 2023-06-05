use rand::{distributions::Alphanumeric, Rng};
use serde::{Deserialize, Serialize};

/// Encrypted value that can be stored safely in
/// any storage medium, including a database.
#[derive(Serialize, Deserialize, Debug)]
pub struct EncryptedValue {
    pub value: Vec<u8>,
    pub nonce: Vec<u8>,
}

/// Generate a random string of any length.
pub fn random_string(len: usize) -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(len)
        .map(char::from)
        .collect()
}
