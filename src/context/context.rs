use crate::context::Environment;
use crate::db::DatabaseConnection;
use crate::error::InternalError;
use std::ops::Deref;

pub struct Context {
    env: Environment,
}

impl Context {
    pub fn env(&self) -> &Environment {
        &self.env
    }

    pub async fn db(&self) -> Result<DatabaseConnection, InternalError> {
        self.env.db.acquire().await
    }
}

pub struct SystemContext(pub Context);

impl Deref for SystemContext {
    type Target = Context;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl SystemContext {
    pub fn new(env: Environment) -> Self {
        SystemContext(Context { env })
    }
}
