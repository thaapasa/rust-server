use crate::context::Environment;
use crate::db::{Database, TransactionalDatabase};
use crate::error::InternalError;
use async_trait::async_trait;
use std::ops::Deref;

pub trait Context {
    fn env(&self) -> &Environment;

    fn db(&mut self) -> &mut Database;

    async fn begin(&mut self) -> Result<TxContext, InternalError>;
}

#[async_trait]
pub trait Transactional {
    async fn commit(mut self) -> Result<(), InternalError>;
    async fn rollback(mut self) -> Result<(), InternalError>;
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

    async fn begin(&mut self) -> Result<TxContext, InternalError> {
        let db = TransactionalDatabase::begin(&mut self.db, None).await?;
        Ok(TxContext {
            env: self.env.clone(),
            db,
            finished: false,
        })
    }
}

pub struct TxContext<'a> {
    env: Environment,
    db: TransactionalDatabase<'a>,
    finished: bool,
}

impl Context for TxContext<'_> {
    fn env(&self) -> &Environment {
        &self.env
    }
    fn db(&mut self) -> &mut Database {
        &mut self.db
    }
    async fn begin(&mut self) -> Result<TxContext, InternalError> {
        let db = TransactionalDatabase::begin(&mut self.db, Some("sp\"1".to_string())).await?;
        Ok(TxContext {
            env: self.env.clone(),
            db,
            finished: false,
        })
    }
}

#[async_trait]
impl Transactional for TxContext<'_> {
    async fn commit(mut self) -> Result<(), InternalError> {
        if self.finished {
            panic!("Transaction already finished")
        }
        self.finished = true;
        self.db.commit().await
    }
    async fn rollback(mut self) -> Result<(), InternalError> {
        if self.finished {
            panic!("Transaction already finished")
        }
        self.finished = true;
        self.db.rollback().await
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
