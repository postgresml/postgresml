use pgrx::*;
use serde::Deserialize;

#[derive(PostgresEnum, Copy, Clone, Eq, PartialEq, Debug, Deserialize)]
#[allow(non_camel_case_types)]
pub enum Status {
    in_progress,
    successful,
    failed,
}

impl std::str::FromStr for Status {
    type Err = ();

    fn from_str(input: &str) -> Result<Status, Self::Err> {
        match input {
            "in_progress" => Ok(Status::in_progress),
            "successful" => Ok(Status::successful),
            "failed" => Ok(Status::failed),
            _ => Err(()),
        }
    }
}

impl std::string::ToString for Status {
    fn to_string(&self) -> String {
        match *self {
            Status::in_progress => "in_progress".to_string(),
            Status::successful => "successful".to_string(),
            Status::failed => "failed".to_string(),
        }
    }
}
