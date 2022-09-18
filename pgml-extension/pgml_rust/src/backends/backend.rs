use pgx::*;
use serde::Deserialize;

#[derive(PostgresEnum, Copy, Clone, Eq, PartialEq, Debug, Deserialize)]
#[allow(non_camel_case_types)]
pub enum Backend {
    xgboost,
    torch,
    lightdbm,
    sklearn,
    smartcore,
    linfa,
}

impl std::str::FromStr for Backend {
    type Err = ();

    fn from_str(input: &str) -> Result<Backend, Self::Err> {
        match input {
            "xgboost" => Ok(Backend::xgboost),
            "torch" => Ok(Backend::torch),
            "lightdbm" => Ok(Backend::lightdbm),
            "sklearn" => Ok(Backend::sklearn),
            "smartcore" => Ok(Backend::smartcore),
            "linfa" => Ok(Backend::linfa),
            _ => Err(()),
        }
    }
}

impl std::string::ToString for Backend {
    fn to_string(&self) -> String {
        match *self {
            Backend::xgboost => "xgboost".to_string(),
            Backend::torch => "torch".to_string(),
            Backend::lightdbm => "lightdbm".to_string(),
            Backend::sklearn => "sklearn".to_string(),
            Backend::smartcore => "smartcore".to_string(),
            Backend::linfa => "linfa".to_string(),
        }
    }
}
