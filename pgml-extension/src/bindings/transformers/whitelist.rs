use anyhow::{bail, Error};
use pgrx::GucSetting;
#[cfg(any(test, feature = "pg_test"))]
use pgrx::{pg_schema, pg_test};
use serde_json::Value;
use std::ffi::CStr;

use crate::config::{PGML_HF_TRUST_REMOTE_CODE, PGML_HF_TRUST_REMOTE_CODE_WHITELIST, PGML_HF_WHITELIST};

/// Verify that the model in the task JSON is allowed based on the huggingface whitelists.
pub fn verify_task(task: &Value) -> Result<(), Error> {
    let task_model = match get_model_name(task) {
        Some(model) => model.to_string(),
        None => return Ok(()),
    };
    let whitelisted_models = config_csv_list(&PGML_HF_WHITELIST);

    let model_is_allowed = whitelisted_models.is_empty() || whitelisted_models.contains(&task_model);
    if !model_is_allowed {
        bail!(
            "model {task_model} is not whitelisted. Consider adding to `pgml.huggingface_whitelist` in postgresql.conf"
        );
    }

    let task_trust = get_trust_remote_code(task);
    let trust_remote_code = PGML_HF_TRUST_REMOTE_CODE.get();

    let trusted_models = config_csv_list(&PGML_HF_TRUST_REMOTE_CODE_WHITELIST);

    let model_is_trusted = trusted_models.is_empty() || trusted_models.contains(&task_model);

    let remote_code_allowed = trust_remote_code && model_is_trusted;
    if !remote_code_allowed && task_trust == Some(true) {
        bail!("model {task_model} is not trusted to run remote code. Consider setting pgml.huggingface_trust_remote_code = 'true' or adding {task_model} to pgml.huggingface_trust_remote_code_whitelist");
    }

    Ok(())
}

fn config_csv_list(csv_list: &GucSetting<Option<&'static CStr>>) -> Vec<String> {
    match csv_list.get() {
        Some(value) => value
            .to_string_lossy()
            .trim_matches('"')
            .split(',')
            .filter_map(|s| if s.is_empty() { None } else { Some(s.to_string()) })
            .collect(),
        None => vec![],
    }
}

fn get_model_name(task: &Value) -> Option<&str> {
    // The JSON key for a model
    static TASK_MODEL_KEY: &str = "model";
    match task {
        Value::Object(map) => map.get(TASK_MODEL_KEY).and_then(|v| {
            if let Value::String(s) = v {
                Some(s.as_str())
            } else {
                None
            }
        }),
        _ => None,
    }
}

fn get_trust_remote_code(task: &Value) -> Option<bool> {
    // The JSON key for the trust remote code flag
    static TASK_REMOTE_CODE_KEY: &str = "trust_remote_code";
    match task {
        Value::Object(map) => {
            map.get(TASK_REMOTE_CODE_KEY)
                .and_then(|v| if let Value::Bool(trust) = v { Some(*trust) } else { None })
        }
        _ => None,
    }
}

#[cfg(any(test, feature = "pg_test"))]
#[pg_schema]
mod tests {
    use super::*;

    use crate::config::set_config;

    // used for copy/pasting a templated string
    macro_rules! json_template {
        () => {
            r#"
            {{
                "task": "task-generation",
                "model": "{}",
                "trust_remote_code": {}
            }}"#
        };
    }

    #[test]
    fn test_get_model_name() {
        let model = "Salesforce/xgen-7b-8k-inst";
        let task_json = format!(json_template!(), model, false);
        let task: Value = serde_json::from_str(&task_json).unwrap();
        assert_eq!(get_model_name(&task), Some(model));
    }

    #[test]
    fn test_get_trust_remote_code_some() {
        for trust_remote_code in [false, true] {
            let task_json = format!(json_template!(), "", trust_remote_code);
            let task: Value = serde_json::from_str(&task_json).unwrap();
            assert_eq!(get_trust_remote_code(&task), Some(trust_remote_code));
        }
    }

    #[test]
    fn test_get_trust_remote_code_none() {
        let task: Value = serde_json::from_str(r#"{ "key": "value" }"#).unwrap();
        assert_eq!(get_trust_remote_code(&task), None);
    }

    #[pg_test]
    fn test_empty_whitelist() {
        let model = "Salesforce/xgen-7b-8k-inst";
        set_config("pgml.huggingface_whitelist", "").unwrap();
        let task_json = format!(json_template!(), model, false);
        let task: Value = serde_json::from_str(&task_json).unwrap();
        assert!(verify_task(&task).is_ok());
    }

    #[pg_test]
    fn test_nonempty_whitelist() {
        let model = "Salesforce/xgen-7b-8k-inst";
        set_config("pgml.huggingface_whitelist", model).unwrap();
        let task_json = format!(json_template!(), model, false);
        let task: Value = serde_json::from_str(&task_json).unwrap();
        assert!(verify_task(&task).is_ok());

        set_config("pgml.huggingface_whitelist", "other_model").unwrap();
        let task_json = format!(json_template!(), model, false);
        let task: Value = serde_json::from_str(&task_json).unwrap();
        assert!(verify_task(&task).is_err());
    }

    #[pg_test]
    fn test_trusted_model() {
        let model = "Salesforce/xgen-7b-8k-inst";
        set_config("pgml.huggingface_whitelist", model).unwrap();
        set_config("pgml.huggingface_trust_remote_code_whitelist", model).unwrap();

        let task_json = format!(json_template!(), model, false);
        let task: Value = serde_json::from_str(&task_json).unwrap();
        assert!(verify_task(&task).is_ok());

        let task_json = format!(json_template!(), model, true);
        let task: Value = serde_json::from_str(&task_json).unwrap();
        assert!(verify_task(&task).is_err());

        set_config("pgml.huggingface_trust_remote_code", "true").unwrap();
        let task_json = format!(json_template!(), model, false);
        let task: Value = serde_json::from_str(&task_json).unwrap();
        assert!(verify_task(&task).is_ok());

        let task_json = format!(json_template!(), model, true);
        let task: Value = serde_json::from_str(&task_json).unwrap();
        assert!(verify_task(&task).is_ok());
    }

    #[pg_test]
    fn test_untrusted_model() {
        let model = "Salesforce/xgen-7b-8k-inst";
        set_config("pgml.huggingface_whitelist", model).unwrap();
        set_config("pgml.huggingface_trust_remote_code_whitelist", "other_model").unwrap();

        let task_json = format!(json_template!(), model, false);
        let task: Value = serde_json::from_str(&task_json).unwrap();
        assert!(verify_task(&task).is_ok());

        let task_json = format!(json_template!(), model, true);
        let task: Value = serde_json::from_str(&task_json).unwrap();
        assert!(verify_task(&task).is_err());

        set_config("pgml.huggingface_trust_remote_code", "true").unwrap();
        let task_json = format!(json_template!(), model, false);
        let task: Value = serde_json::from_str(&task_json).unwrap();
        assert!(verify_task(&task).is_ok());

        let task_json = format!(json_template!(), model, true);
        let task: Value = serde_json::from_str(&task_json).unwrap();
        assert!(verify_task(&task).is_err());
    }
}
