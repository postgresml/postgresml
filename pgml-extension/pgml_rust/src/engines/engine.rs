use pgx::*;
use serde::Deserialize;

#[derive(PostgresEnum, Copy, Clone, Eq, PartialEq, Debug, Deserialize)]
#[allow(non_camel_case_types)]
pub enum Engine {
    xgboost,
    torch,
    lightgbm,
    sklearn,
    smartcore,
    linfa,
}

impl std::str::FromStr for Engine {
    type Err = ();

    fn from_str(input: &str) -> Result<Engine, Self::Err> {
        match input {
            "xgboost" => Ok(Engine::xgboost),
            "torch" => Ok(Engine::torch),
            "lightgbm" => Ok(Engine::lightgbm),
            "sklearn" => Ok(Engine::sklearn),
            "smartcore" => Ok(Engine::smartcore),
            "linfa" => Ok(Engine::linfa),
            _ => Err(()),
        }
    }
}

impl std::string::ToString for Engine {
    fn to_string(&self) -> String {
        match *self {
            Engine::xgboost => "xgboost".to_string(),
            Engine::torch => "torch".to_string(),
            Engine::lightgbm => "lightgbm".to_string(),
            Engine::sklearn => "sklearn".to_string(),
            Engine::smartcore => "smartcore".to_string(),
            Engine::linfa => "linfa".to_string(),
        }
    }
}
