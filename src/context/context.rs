use std::future::Future;

use crate::context::Environment;
use crate::db::{DatabaseAccess, DatabasePool, TransactionalDatabase};
use crate::error::InternalError;

pub trait Context: Send + Sync {
    fn env(&self) -> &Environment;

    fn db(&mut self) -> &mut impl DatabaseAccess;

    fn begin(
        &mut self,
    ) -> impl Future<Output = Result<impl Context + Transactional, InternalError>> + Send;
}

pub trait Transactional: Send + Sync {
    fn commit(self) -> impl Future<Output = Result<(), InternalError>> + Send;
    fn rollback(self) -> impl Future<Output = Result<(), InternalError>> + Send;
}

pub struct RootContext {
    env: Environment,
    pool: DatabasePool,
}

impl RootContext {
    pub fn new(env: Environment) -> Self {
        let pool = env.db_pool.clone();
        RootContext { env, pool }
    }
}

impl Context for RootContext {
    fn env(&self) -> &Environment {
        &self.env
    }

    fn db(&mut self) -> &mut impl DatabaseAccess {
        &mut self.pool
    }

    async fn begin(&mut self) -> Result<impl Context + Transactional, InternalError> {
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

impl Context for TxContext<'_> {
    fn env(&self) -> &Environment {
        &self.env
    }

    fn db(&mut self) -> &mut impl DatabaseAccess {
        &mut self.db
    }

    async fn begin(&mut self) -> Result<impl Context + Transactional, InternalError> {
        let tx = self.db.begin().await?;
        Ok(TxContext {
            env: self.env.clone(),
            db: tx,
        })
    }
}

impl Transactional for TxContext<'_> {
    async fn commit(self) -> Result<(), InternalError> {
        self.db.commit().await
    }

    async fn rollback(self) -> Result<(), InternalError> {
        self.db.rollback().await
    }
}
