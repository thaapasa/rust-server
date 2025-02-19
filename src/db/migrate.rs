use crate::context::Environment;
use crate::error::InternalError;
use sqlx::migrate::Migrator;
use sqlx::PgPool;
use tracing::info;

static MIGRATOR: Migrator = sqlx::migrate!("./migrations");

pub async fn run_db_migrations(env: &Environment) -> Result<(), InternalError> {
    let pool = PgPool::connect(&env.config.database.url)
        .await
        .map_err(InternalError::from)?;
    info!("Running DB migrations");
    MIGRATOR.run(&pool).await.map_err(InternalError::from)?;
    info!("DB migrations complete");
    Ok(())
}
