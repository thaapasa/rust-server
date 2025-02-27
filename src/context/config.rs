use config::builder::DefaultState;
use config::{Config as ConfigCrate, ConfigBuilder};
use config::{ConfigError, Environment as ConfigEnvironment, File};
use serde::Deserialize;

use crate::error::InternalError;

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub environment_name: String,
    pub server: ServerSettings,
    pub database: DatabaseSettings,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ServerSettings {
    pub port: u16,
}

#[derive(Debug, Deserialize, Clone)]
pub struct DatabaseSettings {
    pub url: String,
}

impl Config {
    pub fn new_from_file(file_name: String) -> Result<Self, InternalError> {
        let builder = ConfigCrate::builder().add_source(File::with_name(&file_name));
        Self::build(builder)
    }

    pub fn build(builder: ConfigBuilder<DefaultState>) -> Result<Self, InternalError> {
        let settings = builder
            .add_source(ConfigEnvironment::default())
            .build()
            .map_err(InternalError::from)?;

        settings.try_deserialize().map_err(Into::into)
    }
}

impl From<ConfigError> for InternalError {
    fn from(value: ConfigError) -> Self {
        InternalError::message(format!("{value:?}"))
    }
}
