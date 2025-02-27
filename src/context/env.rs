use sqlx::{Pool, Postgres};
use std::env;
use tracing::debug;

use crate::context::Config;
use crate::db::Database;
use crate::error::InternalError;

#[derive(Debug, Clone)]
pub struct Environment {
    pub config: Config,
    pub db: Database,
}

impl Environment {
    pub async fn init() -> Result<Self, InternalError> {
        // Read config path from the environment variable or use a default
        let config_path = env::var("CONFIG_FILE").unwrap_or_else(|_| "setting.toml".to_string());
        debug!("Reading configuration from {config_path}");
        let config = Config::new_from_file(config_path)?;
        Self::init_with_config(config).await
    }

    pub async fn init_with_config(config: Config) -> Result<Self, InternalError> {
        let db = Database::init_pool(&config.database.url).await?;
        Ok(Environment { config, db })
    }

    pub fn db_pool(&self) -> &Pool<Postgres> {
        match &self.db {
            Database::DbPool(pool) => pool,
            _ => panic!("Environment setup failure; DB is not pool"),
        }
    }

    pub async fn db_conn(&self) -> Result<Database, InternalError> {
        self.db.connection().await
    }
}
