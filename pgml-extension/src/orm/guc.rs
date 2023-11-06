use core::ffi::CStr;
use lazy_static::*;
use pgrx::*;
use std::collections::BTreeMap;
use std::option::Option;
use std::path::PathBuf;

struct Gucs {
    pgml_cache: GucSetting<Option<&'static CStr>>,
    hf_hub_offline: GucSetting<bool>,
}

lazy_static! {
    static ref GUCS: Gucs = Gucs {
        pgml_cache: GucSetting::<Option<&'static CStr>>::new(None),
        hf_hub_offline: GucSetting::<bool>::new(false),
    };
}

pub fn pgml_cache_guc() -> Option<String> {
    let v = GUCS.pgml_cache.get();
    v.map(|v| v.to_string_lossy().to_string())
}

/// Creates map of ENVs to be set before staring huggingface python APIs.
pub fn gen_hf_env_map() -> BTreeMap<&'static str, String> {
    let mut map = BTreeMap::new();

    let base_path: PathBuf;
    match GUCS.pgml_cache.get() {
        Some(value) => {
            let value = value.to_string_lossy().to_string();
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
    map.insert("HF_HOME", format!("{}", hf_home.display()));

    // SENTENCE_TRANSFORMERS_HOME is for sentence transformers
    let mut torch_home = base_path.clone();
    torch_home.push("torch");
    map.insert(
        "SENTENCE_TRANSFORMERS_HOME",
        format!("{}", torch_home.display()),
    );

    map.insert(
        "HF_HUB_OFFLINE",
        if GUCS.hf_hub_offline.get() {
            "TRUE".to_string()
        } else {
            "FALSE".to_string()
        },
    );

    return map;
}
