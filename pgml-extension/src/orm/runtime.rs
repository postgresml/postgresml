use pgrx::*;
use serde::Deserialize;

#[derive(PostgresEnum, Copy, Clone, Eq, PartialEq, Debug, Deserialize)]
#[allow(non_camel_case_types)]
pub enum Runtime {
    python,
    rust,
    openai,
}

impl std::str::FromStr for Runtime {
    type Err = ();

    fn from_str(input: &str) -> Result<Runtime, Self::Err> {
        match input {
            "python" => Ok(Runtime::python),
            "rust" => Ok(Runtime::rust),
            "openai" => Ok(Runtime::openai),
            _ => Err(()),
        }
    }
}

impl std::string::ToString for Runtime {
    fn to_string(&self) -> String {
        match *self {
            Runtime::python => "python".to_string(),
            Runtime::rust => "rust".to_string(),
            Runtime::openai => "openai".to_string(),
        }
    }
}
