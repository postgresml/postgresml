use std::fmt;

use pyo3::PyErr;

use super::whitelist;

#[derive(Debug)]
pub enum Error {
    Serde(serde_json::Error),
    Python(PyErr),
    Model(whitelist::Error),
    Data(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Python(e) => write!(f, "{e}"),
            Error::Model(e) => write!(f, "{e}"),
            Error::Serde(e) => write!(f, "{e}"),
            Error::Data(e) => write!(f, "{e}"),
        }
    }
}

impl std::error::Error for Error {}

impl From<PyErr> for Error {
    fn from(value: PyErr) -> Self {
        Self::Python(value)
    }
}

impl From<whitelist::Error> for Error {
    fn from(value: whitelist::Error) -> Self {
        Self::Model(value)
    }
}

impl From<serde_json::Error> for Error {
    fn from(value: serde_json::Error) -> Self {
        Self::Serde(value)
    }
}
