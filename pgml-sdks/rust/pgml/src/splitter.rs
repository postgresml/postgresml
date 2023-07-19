use pgml_macros::{custom_derive, custom_methods};

use crate::{
    models,
    types::{DateTime, Json},
};

#[cfg(feature = "javascript")]
use crate::languages::javascript::*;

#[derive(custom_derive, Debug, Clone)]
pub struct Splitter {
    pub id: i64,
    pub created_at: DateTime,
    pub name: String,
    pub parameters: Json,
}

#[custom_methods(get_id, get_created_at, get_name, get_parameters)]
impl Splitter {
    pub fn get_id(&self) -> i64 {
        self.id
    }

    pub fn get_created_at(&self) -> DateTime {
        self.created_at.clone()
    }

    pub fn get_name(&self) -> String {
        self.name.clone()
    }

    pub fn get_parameters(&self) -> Json {
        self.parameters.clone()
    }
}

impl From<models::Splitter> for Splitter {
    fn from(m: models::Splitter) -> Self {
        Self {
            id: m.id,
            created_at: m.created_at,
            name: m.name,
            parameters: m.parameters,
        }
    }
}
