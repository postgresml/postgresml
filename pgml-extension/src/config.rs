use std::collections::BTreeMap;
use std::env;
use std::ffi::CStr;
use std::path::PathBuf;

#[cfg(any(test, feature = "pg_test"))]
use pgrx::{pg_schema, pg_test};
use pgrx_pg_sys::AsPgCStr;

// GUC variables for huggingface
pub static CONFIG_CACHE: &str = "pgml.cache";
pub static CONFIG_OFFLINE: &str = "pgml.offline";

// environment variables for huggingface and sentence-transformers
static ENV_HF_HOME: &str = "HF_HOME";
static ENV_SENTENCE_TRANSFORMERS_HOME: &str = "SENTENCE_TRANSFORMERS_HOME";
static ENV_HF_HUB_OFFLINE: &str = "HF_HUB_OFFLINE";

pub fn get_config(name: &str) -> Option<String> {
    // SAFETY: name is not null because it is a Rust reference.
    let ptr = unsafe { pgrx_pg_sys::GetConfigOption(name.as_pg_cstr(), true, false) };
    (!ptr.is_null()).then(move || {
        // SAFETY: assuming pgrx_pg_sys is providing a valid, null terminated pointer.
        unsafe { CStr::from_ptr(ptr) }.to_string_lossy().to_string()
    })
}

#[cfg(any(test, feature = "pg_test"))]
pub fn set_config(name: &str, value: &str) -> Result<(), pgrx::spi::Error> {
    // using Spi::run instead of pgrx_pg_sys interface because it seems much easier,
    // especially since this is just for testing
    let query = format!("SELECT set_config('{name}', '{value}', false);");
    pgrx::Spi::run(&query)
}

/// Creates map of ENVs to be set before staring huggingface python APIs.
pub fn gen_hf_env_map() -> BTreeMap<&'static str, String> {
    let mut map = BTreeMap::new();

    let base_path: PathBuf;

    match get_config(CONFIG_CACHE) {
        Some(value) => {
            base_path = PathBuf::from(value);
        }
        None => {
            return map;
        }
    }

    // HF_HOME is for huggingface
    // https://huggingface.co/docs/huggingface_hub/package_reference/environment_variables#hfhome
    let mut hf_home = base_path.clone();
    hf_home.push("huggingface");
    map.insert(ENV_HF_HOME, format!("{}", hf_home.display()));

    // SENTENCE_TRANSFORMERS_HOME is for sentence-transformers
    let mut torch_home = base_path.clone();
    torch_home.push("torch");
    map.insert(
        ENV_SENTENCE_TRANSFORMERS_HOME,
        format!("{}", torch_home.display()),
    );

    let offline_value: String;
    match get_config(CONFIG_OFFLINE) {
        Some(value) => {
            if value.is_empty() {
                offline_value = String::from("FALSE");
            } else {
                offline_value = String::from("TRUE");
            }
        }
        None => {
            offline_value = String::from("TRUE");
        }
    }
    map.insert(ENV_HF_HUB_OFFLINE, offline_value);

    return map;
}

// Called before hugginface python APIs. Setup ENVs for HuggingFace. See
// https://huggingface.co/docs/huggingface_hub/package_reference/environment_variables
pub fn set_env() {
    let envs_to_apply = gen_hf_env_map();

    // Set the env
    for (k, v) in &envs_to_apply {
        if v.trim().is_empty() {
            env::remove_var(k);
        } else {
            env::set_var(k, v);
        }
    }
}

#[cfg(any(test, feature = "pg_test"))]
#[pg_schema]
mod tests {
    use super::*;

    #[pg_test]
    fn read_config_max_connections() {
        let name = "max_connections";
        assert_eq!(get_config(name), Some("100".into()));
    }

    #[pg_test]
    fn read_pgml_huggingface_whitelist() {
        let name = "pgml.huggingface_whitelist";
        let value = "meta-llama/Llama-2-7b";
        set_config(name, value).unwrap();
        assert_eq!(get_config(name), Some(value.into()));
    }

    #[pg_test]
    fn test_set_env() {
        use crate::config::set_config;

        let tmp_path: &str = "/tmp/pgml";

        set_config(CONFIG_CACHE, tmp_path).unwrap();

        set_env();
        let _ = crate::bindings::python::activate();

        let base_path: PathBuf;

        match get_config(CONFIG_CACHE) {
            Some(value) => {
                base_path = PathBuf::from(value);
                let base_path = base_path.display();

                assert_eq!(
                    env::var(ENV_HF_HOME).unwrap(),
                    format!("{}/huggingface", base_path)
                );
                assert_eq!(
                    env::var(ENV_SENTENCE_TRANSFORMERS_HOME).unwrap(),
                    format!("{}/torch", base_path)
                );
                assert_eq!(
                    env::var(ENV_HF_HOME).unwrap(),
                    format!("{}/huggingface", tmp_path)
                );
                assert_eq!(
                    env::var(ENV_SENTENCE_TRANSFORMERS_HOME).unwrap(),
                    format!("{}/torch", tmp_path)
                );
            }
            None => {
                assert!(env::var(ENV_HF_HOME).is_err());
                assert!(env::var(ENV_SENTENCE_TRANSFORMERS_HOME).is_err());
            }
        }
    }
}
