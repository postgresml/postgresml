use anyhow::{bail, Error};
#[cfg(any(test, feature = "pg_test"))]
use pgrx::{pg_schema, pg_test};
use serde_json::Value;

use crate::config::get_config;

static CONFIG_HF_WHITELIST: &str = "pgml.huggingface_whitelist";
static CONFIG_HF_TRUST_REMOTE_CODE_BOOL: &str = "pgml.huggingface_trust_remote_code";
static CONFIG_HF_TRUST_WHITELIST: &str = "pgml.huggingface_trust_remote_code_whitelist";

/// Verify that the model in the task JSON is allowed based on the huggingface whitelists.
pub fn verify_task(task: &Value) -> Result<(), Error> {
    let task_model = match get_model_name(task) {
        Some(model) => model.to_string(),
        None => return Ok(()),
    };
    let whitelisted_models = config_csv_list(CONFIG_HF_WHITELIST);

    let model_is_allowed = whitelisted_models.is_empty() || whitelisted_models.contains(&task_model);
    if !model_is_allowed {
        bail!("model {task_model} is not whitelisted. Consider adding to {CONFIG_HF_WHITELIST} in postgresql.conf");
    }

    let task_trust = get_trust_remote_code(task);
    let trust_remote_code = get_config(CONFIG_HF_TRUST_REMOTE_CODE_BOOL)
        .map(|v| v == "true")
        .unwrap_or(true);

    let trusted_models = config_csv_list(CONFIG_HF_TRUST_WHITELIST);

    let model_is_trusted = trusted_models.is_empty() || trusted_models.contains(&task_model);

    let remote_code_allowed = trust_remote_code && model_is_trusted;
    if !remote_code_allowed && task_trust == Some(true) {
        bail!("model {task_model} is not trusted to run remote code. Consider setting {CONFIG_HF_TRUST_REMOTE_CODE_BOOL} = 'true' or adding {task_model} to {CONFIG_HF_TRUST_WHITELIST}");
    }

    Ok(())
}

fn config_csv_list(name: &str) -> Vec<String> {
    match get_config(name) {
        Some(value) => value
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
        set_config(CONFIG_HF_WHITELIST, "").unwrap();
        let task_json = format!(json_template!(), model, false);
        let task: Value = serde_json::from_str(&task_json).unwrap();
        assert!(verify_task(&task).is_ok());
    }

    #[pg_test]
    fn test_nonempty_whitelist() {
        let model = "Salesforce/xgen-7b-8k-inst";
        set_config(CONFIG_HF_WHITELIST, model).unwrap();
        let task_json = format!(json_template!(), model, false);
        let task: Value = serde_json::from_str(&task_json).unwrap();
        assert!(verify_task(&task).is_ok());

        set_config(CONFIG_HF_WHITELIST, "other_model").unwrap();
        let task_json = format!(json_template!(), model, false);
        let task: Value = serde_json::from_str(&task_json).unwrap();
        assert!(verify_task(&task).is_err());
    }

    #[pg_test]
    fn test_trusted_model() {
        let model = "Salesforce/xgen-7b-8k-inst";
        set_config(CONFIG_HF_WHITELIST, model).unwrap();
        set_config(CONFIG_HF_TRUST_WHITELIST, model).unwrap();

        let task_json = format!(json_template!(), model, false);
        let task: Value = serde_json::from_str(&task_json).unwrap();
        assert!(verify_task(&task).is_ok());

        let task_json = format!(json_template!(), model, true);
        let task: Value = serde_json::from_str(&task_json).unwrap();
        assert!(verify_task(&task).is_ok());

        set_config(CONFIG_HF_TRUST_REMOTE_CODE_BOOL, "true").unwrap();
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
        set_config(CONFIG_HF_WHITELIST, model).unwrap();
        set_config(CONFIG_HF_TRUST_WHITELIST, "other_model").unwrap();

        let task_json = format!(json_template!(), model, false);
        let task: Value = serde_json::from_str(&task_json).unwrap();
        assert!(verify_task(&task).is_ok());

        let task_json = format!(json_template!(), model, true);
        let task: Value = serde_json::from_str(&task_json).unwrap();
        assert!(verify_task(&task).is_err());

        set_config(CONFIG_HF_TRUST_REMOTE_CODE_BOOL, "true").unwrap();
        let task_json = format!(json_template!(), model, false);
        let task: Value = serde_json::from_str(&task_json).unwrap();
        assert!(verify_task(&task).is_ok());

        let task_json = format!(json_template!(), model, true);
        let task: Value = serde_json::from_str(&task_json).unwrap();
        assert!(verify_task(&task).is_err());
    }
}
