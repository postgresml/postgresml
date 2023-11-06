use std::collections::BTreeMap;
use std::path::PathBuf;

use crate::config::get_config;

pub static CONFIG_CACHE: &str = "pgml.cache";
pub static CONFIG_OFFLINE: &str = "pgml.offline";

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
    map.insert("HF_HOME", format!("{}", hf_home.display()));

    // SENTENCE_TRANSFORMERS_HOME is for sentence transformers
    let mut torch_home = base_path.clone();
    torch_home.push("torch");
    map.insert(
        "SENTENCE_TRANSFORMERS_HOME",
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
    map.insert("HF_HUB_OFFLINE", offline_value);

    return map;
}
