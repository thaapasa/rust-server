use crate::context::{Config, Context, ContextImpl, Environment};
use crate::db::run_db_migrations;
use crate::error::InternalError;
use crate::logging::configure_logging;
use config::Config as ConfigCrate;
use sql::sql;

pub struct TestEnvironment {
    pub env: Environment,
}

pub fn test_config() -> Result<Config, InternalError> {
    Config::build(
        ConfigCrate::builder()
            .set_default("environment_name", "Test")?
            .set_default("server.port", "6100")?
            .set_default(
                "database.url",
                "postgresql://postgres:postgres@localhost:6110/postgres",
            )?,
    )
}

impl TestEnvironment {
    pub async fn init() -> TestEnvironment {
        configure_logging();
        let config = test_config().unwrap();
        let env = Environment::init_with_config(config).await.unwrap();
        let env = TestEnvironment { env };
        env.init_db().await;
        env
    }

    async fn init_db(&self) {
        let mut ctx = self.ctx().await;
        reset_db(&mut ctx).await.unwrap();
        run_db_migrations(&self.env).await.unwrap();
    }

    pub async fn ctx(&self) -> impl Context {
        ContextImpl::new(self.env.clone()).await.unwrap()
    }
}

async fn reset_db(ctx: &mut impl Context) -> Result<(), InternalError> {
    ctx.db().execute(sql!("DROP SCHEMA public CASCADE")).await?;
    ctx.db().execute(sql!("CREATE SCHEMA public")).await?;
    Ok(())
}
