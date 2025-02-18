use config::Config as ConfigCrate;
use config::{ConfigError, Environment as ConfigEnvironment, File};
use serde::Deserialize;

use crate::error::InternalError;

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub environment_name: String,
    pub server: ServerSettings,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ServerSettings {
    pub port: u16,
}

impl Config {
    pub fn new_from_file(file_name: String) -> Result<Self, InternalError> {
        let settings = ConfigCrate::builder()
            .add_source(File::with_name(&file_name))
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
