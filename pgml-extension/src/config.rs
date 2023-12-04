use std::collections::BTreeMap;
use std::env;
use std::ffi::CStr;
use std::path::PathBuf;

use lazy_static::lazy_static;
use pgrx::guc::GucFlags;
#[cfg(any(test, feature = "pg_test"))]
use pgrx::{pg_schema, pg_test};
use pgrx::{GucContext, GucRegistry, GucSetting};
use pgrx_pg_sys::AsPgCStr;

// GUC variables for huggingface
pub static CONFIG_CACHE: &str = "pgml.cache";
pub static CONFIG_OFFLINE: &str = "pgml.offline";

// environment variables for huggingface and sentence-transformers
static ENV_HF_HOME: &str = "HF_HOME";
static ENV_SENTENCE_TRANSFORMERS_HOME: &str = "SENTENCE_TRANSFORMERS_HOME";
static ENV_HF_HUB_OFFLINE: &str = "HF_HUB_OFFLINE";

struct Guc {
    pgml_cache: GucSetting<Option<&'static CStr>>,
    pgml_offline: GucSetting<bool>,
}

lazy_static! {
    static ref GUC: Guc = Guc {
        pgml_cache: GucSetting::<Option<&'static CStr>>::new(None),
        pgml_offline: GucSetting::<bool>::new(false),
    };
}

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

    if let Some(value) = GUC.pgml_cache.get() {
        let base_path: PathBuf;
        base_path = PathBuf::from(value.to_str().unwrap_or_default());

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
    }

    map.insert(
        ENV_HF_HUB_OFFLINE,
        if GUC.pgml_offline.get() {
            String::from("TRUE")
        } else {
            String::from("FALSE")
        },
    );

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

pub fn init_gucs() {
    GucRegistry::define_string_guc(
        CONFIG_CACHE,
        "pgml cache directory",
        "This directory will serve as huggingface home and pytorch cache",
        &GUC.pgml_cache,
        GucContext::Suset,
        GucFlags::default(),
    );

    GucRegistry::define_bool_guc(
        CONFIG_OFFLINE,
        "Set the huggingface hub to offline mode, same as HF_HUB_OFFLINE",
        "If set, no HTTP calls will be made when trying to fetch files. Only files that are already cached will be accessed. This is useful in case your network is slow and you don’t care about having absolutely the latest version of a file.",
        &GUC.pgml_offline,
        GucContext::Suset,
        GucFlags::default(),
    );
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

        match GUC.pgml_cache.get() {
            Some(value) => {
                base_path = PathBuf::from(value.to_str().unwrap_or_default());
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

        assert_eq!(env::var(ENV_HF_HUB_OFFLINE).unwrap(), "FALSE");
    }
}
