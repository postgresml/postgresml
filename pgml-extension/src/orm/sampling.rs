use pgrx::*;
use serde::Deserialize;

#[derive(PostgresEnum, Copy, Clone, Eq, PartialEq, Debug, Deserialize)]
#[allow(non_camel_case_types)]
pub enum Sampling {
    random,
    last,
}

impl std::str::FromStr for Sampling {
    type Err = ();

    fn from_str(input: &str) -> Result<Sampling, Self::Err> {
        match input {
            "random" => Ok(Sampling::random),
            "last" => Ok(Sampling::last),
            _ => Err(()),
        }
    }
}

impl std::string::ToString for Sampling {
    fn to_string(&self) -> String {
        match *self {
            Sampling::random => "random".to_string(),
            Sampling::last => "last".to_string(),
        }
    }
}
