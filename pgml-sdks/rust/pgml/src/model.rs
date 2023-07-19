use pgml_macros::{custom_derive, custom_methods};

use crate::{
    models,
    types::{DateTime, Json},
};

#[cfg(feature = "javascript")]
use crate::languages::javascript::*;

#[derive(custom_derive, Debug, Clone)]
pub struct Model {
    pub id: i64,
    pub created_at: DateTime,
    pub task: String,
    pub name: String,
    pub source: String,
    pub parameters: Json,
}

#[custom_methods(get_id, get_created_at, get_task, get_name, get_source, get_parameters)]
impl Model {
    pub fn get_id(&self) -> i64 {
        self.id
    }

    pub fn get_created_at(&self) -> DateTime {
        self.created_at.clone()
    }

    pub fn get_task(&self) -> String {
        self.task.clone()
    }

    pub fn get_name(&self) -> String {
        self.name.clone()
    }

    pub fn get_source(&self) -> String {
        self.source.clone()
    }

    pub fn get_parameters(&self) -> Json {
        self.parameters.clone()
    }
}

impl From<models::Model> for Model {
    fn from(m: models::Model) -> Self {
        Self {
            id: m.id,
            created_at: m.created_at,
            task: m.task,
            name: m.name,
            source: m.source,
            parameters: m.parameters,
        }
    }
}
