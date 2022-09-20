pub mod engine;
pub mod sklearn;
pub mod smartcore;
pub mod xgboost;

use serde_json;

pub type Hyperparams = serde_json::Map<std::string::String, serde_json::Value>;
