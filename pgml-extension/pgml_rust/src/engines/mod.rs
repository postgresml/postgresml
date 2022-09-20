pub mod engine;
pub mod sklearn;
pub mod smartcore;
pub mod xgboost;

pub trait FromJSON {
    fn from_json(value: serde_json::value::Value) -> Self;
}
