use pgrx::*;
use serde::Deserialize;

#[derive(PostgresEnum, Copy, Clone, Eq, PartialEq, Debug, Deserialize)]
#[allow(non_camel_case_types)]
pub enum Search {
    grid,
    random,
}

impl std::str::FromStr for Search {
    type Err = ();

    fn from_str(input: &str) -> Result<Search, Self::Err> {
        match input {
            "grid" => Ok(Search::grid),
            "random" => Ok(Search::random),
            _ => Err(()),
        }
    }
}

impl std::string::ToString for Search {
    fn to_string(&self) -> String {
        match *self {
            Search::grid => "grid".to_string(),
            Search::random => "random".to_string(),
        }
    }
}
