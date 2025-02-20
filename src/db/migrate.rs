use crate::context::Environment;
use crate::error::InternalError;
use sqlx::migrate::Migrator;
use tracing::info;

static MIGRATOR: Migrator = sqlx::migrate!("./migrations");

pub async fn run_db_migrations(env: &Environment) -> Result<(), InternalError> {
    let pool = env.db_pool();

    info!("Running DB migrations");
    MIGRATOR.run(pool).await.map_err(InternalError::from)?;
    info!("DB migrations complete");
    Ok(())
}
