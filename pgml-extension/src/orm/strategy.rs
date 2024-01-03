use pgrx::*;
use serde::Deserialize;

#[derive(PostgresEnum, Copy, Clone, Eq, PartialEq, Debug, Deserialize)]
#[allow(non_camel_case_types)]
pub enum Strategy {
    new_score,
    best_score,
    most_recent,
    rollback,
    specific,
}

impl std::str::FromStr for Strategy {
    type Err = ();

    fn from_str(input: &str) -> Result<Strategy, Self::Err> {
        match input {
            "new_score" => Ok(Strategy::new_score),
            "best_score" => Ok(Strategy::best_score),
            "most_recent" => Ok(Strategy::most_recent),
            "rollback" => Ok(Strategy::rollback),
            "specific" => Ok(Strategy::rollback),
            _ => Err(()),
        }
    }
}

impl std::string::ToString for Strategy {
    fn to_string(&self) -> String {
        match *self {
            Strategy::new_score => "new_score".to_string(),
            Strategy::best_score => "best_score".to_string(),
            Strategy::most_recent => "most_recent".to_string(),
            Strategy::rollback => "rollback".to_string(),
            Strategy::specific => "specific".to_string(),
        }
    }
}
