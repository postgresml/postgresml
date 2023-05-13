use aes_gcm::{
    aead::Aead,
    Aes256Gcm,
    KeyInit,
    Nonce, // Or `Aes128Gcm`
};
use anyhow::anyhow;
use base64::{engine::general_purpose, Engine as _};
use rand::{distributions::Alphanumeric, Rng};
use serde::{Deserialize, Serialize};
use sqlx::postgres::types::PgInterval;
use time::Duration;

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

/// Encrypt a value using a AES-256-GCM.
///
/// The provided key must be 256-bits, or 32 characters long.
pub fn encrypt(key: &str, value: &str) -> anyhow::Result<String> {
    let nonce = rand::thread_rng().gen::<[u8; 12]>();
    let key = general_purpose::STANDARD
        .decode(key)
        .expect("base64 decode failed on key");

    let cipher = match Aes256Gcm::new_from_slice(&key) {
        Ok(cipher) => cipher,
        Err(err) => return Err(anyhow::anyhow!("key error: {:?}", err)),
    };

    let nonce = Nonce::from_slice(&nonce);
    let encrypted = match cipher.encrypt(&nonce, value.as_bytes()) {
        Ok(encrypted) => encrypted,
        Err(err) => return Err(anyhow::anyhow!("encryption error: {:?}", err)),
    };

    Ok(serde_json::to_string(&EncryptedValue {
        value: encrypted,
        nonce: nonce.to_vec(),
    })?)
}

/// Take care of unusual error casting that is required.
pub fn pg_duration(dur: &Duration) -> anyhow::Result<PgInterval> {
    PgInterval::try_from(dur.clone()).map_err(|err| anyhow!(err))
}

/// Decrypt a value using AES-256-GCM.
///
/// The provided key must be 256-bits, or 32 characters long.
pub fn decrypt(key: &str, value: &str) -> anyhow::Result<String> {
    let value: EncryptedValue = serde_json::from_str(value)?;
    let key = general_purpose::STANDARD
        .decode(key)
        .expect("base64 decode failed on key");

    let cipher = match Aes256Gcm::new_from_slice(&key) {
        Ok(cipher) => cipher,
        Err(err) => return Err(anyhow::anyhow!("key error: {:?}", err)),
    };

    let nonce = Nonce::from_slice(&value.nonce);
    let decrpyted = match cipher.decrypt(&nonce, value.value.as_slice()) {
        Ok(value) => value,
        Err(err) => return Err(anyhow::anyhow!("decryption error: {:?}", err)),
    };

    Ok(String::from_utf8(decrpyted)?)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_encrypt() {
        let key = "CED3lXZ4voSifPakjydU9cUxgD5xYTrkJUpqvX1RBUA=";
        let enc = encrypt(key, "test value").unwrap();
        let dec = decrypt(key, &enc).unwrap();

        assert_eq!(dec.as_str(), "test value");
    }
}
