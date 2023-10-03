use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Default, Clone)]
pub struct Javascript {
    #[serde(default = "Javascript::default_additional_paths")]
    pub additional_paths: Vec<String>,
}

impl Javascript {
    fn default_additional_paths() -> Vec<String> {
        vec![]
    }
}

#[derive(Serialize, Deserialize, Default, Clone)]
pub struct Config {
    pub javascript: Javascript,
}

impl Config {
    pub fn from_path(path: &str) -> anyhow::Result<Config> {
        let config_str = std::fs::read_to_string(path)?;
        let config: Config = toml::from_str(&config_str)?;
        Ok(config)
    }

    pub fn load() -> Config {
        match Self::from_path("pgml-components.toml") {
            Ok(config) => config,
            Err(_) => Config::default(),
        }
    }
}
