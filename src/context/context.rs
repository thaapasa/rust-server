use crate::context::Environment;
use crate::db::Database;
use crate::error::InternalError;
use std::ops::Deref;

pub trait Context {
    fn env(&self) -> &Environment;

    fn db(&mut self) -> &mut Database;
}

pub struct ContextImpl {
    env: Environment,
    db: Database,
}

impl Context for ContextImpl {
    fn env(&self) -> &Environment {
        &self.env
    }

    fn db(&mut self) -> &mut Database {
        &mut self.db
    }
}

pub struct SystemContext(pub ContextImpl);

impl Deref for SystemContext {
    type Target = ContextImpl;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl SystemContext {
    pub async fn new(env: Environment) -> Result<Self, InternalError> {
        let db = env.db_conn().await?;
        Ok(SystemContext(ContextImpl { env, db }))
    }
}
