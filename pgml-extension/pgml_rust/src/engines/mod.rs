use anyhow::Result;

pub mod engine;
pub mod sklearn;
pub mod smartcore;
pub mod xgboost;

pub trait FromJSON {
    fn from_json(value: &serde_json::Map<std::string::String, serde_json::Value>) -> Result<Self> where Self: Sized;
}
