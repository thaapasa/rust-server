use async_trait::async_trait;

use crate::context::Environment;
use crate::db::{DatabaseAccess, DatabasePool, TransactionalDatabase};
use crate::error::InternalError;

#[async_trait]
pub trait Context: Send + Sync {
    fn env(&self) -> &Environment;

    fn db(&mut self) -> &mut impl DatabaseAccess;

    async fn begin(&mut self) -> Result<TxContext, InternalError>;
}

#[async_trait]
pub trait Transactional: Send + Sync {
    async fn commit(self) -> Result<(), InternalError>;
    async fn rollback(self) -> Result<(), InternalError>;
}

pub struct ContextImpl {
    env: Environment,
    pool: DatabasePool,
}

impl ContextImpl {
    pub async fn new(env: Environment) -> Result<Self, InternalError> {
        let pool = env.db_pool.clone();
        Ok(ContextImpl { env, pool })
    }
}

#[async_trait]
impl Context for ContextImpl {
    fn env(&self) -> &Environment {
        &self.env
    }

    fn db(&mut self) -> &mut impl DatabaseAccess {
        &mut self.pool
    }

    async fn begin(&mut self) -> Result<TxContext, InternalError> {
        let db = TransactionalDatabase::begin_from_pool(&self.pool).await?;
        Ok(TxContext {
            env: self.env.clone(),
            db,
        })
    }
}

pub struct TxContext<'a> {
    env: Environment,
    db: TransactionalDatabase<'a>,
}

#[async_trait]
impl Context for TxContext<'_> {
    fn env(&self) -> &Environment {
        &self.env
    }
    fn db(&mut self) -> &mut impl DatabaseAccess {
        &mut self.db
    }

    async fn begin(&mut self) -> Result<TxContext, InternalError> {
        let tx = self.db.begin().await?;
        Ok(TxContext {
            env: self.env.clone(),
            db: tx,
        })
    }
}

#[async_trait]
impl Transactional for TxContext<'_> {
    async fn commit(self) -> Result<(), InternalError> {
        self.db.commit().await
    }
    async fn rollback(self) -> Result<(), InternalError> {
        self.db.rollback().await
    }
}
