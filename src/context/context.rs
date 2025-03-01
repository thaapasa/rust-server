use async_trait::async_trait;

use crate::context::Environment;
use crate::db::{Database, DatabaseAccess, TransactionalDatabase};
use crate::db::Database::{DbConnection, DbPool};
use crate::error::InternalError;

pub trait Context {
    fn env(&self) -> &Environment;

    fn db(&mut self) -> &mut impl DatabaseAccess;

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

impl ContextImpl {
    pub async fn new(env: Environment) -> Result<Self, InternalError> {
        let db = env.db_conn().await?;
        Ok(ContextImpl { env, db })
    }
}

impl Context for ContextImpl {
    fn env(&self) -> &Environment {
        &self.env
    }

    fn db(&mut self) -> &mut impl DatabaseAccess {
        &mut self.db
    }

    async fn begin(&mut self) -> Result<TxContext, InternalError> {
        let db = match self.db {
            DbPool(ref pool) => TransactionalDatabase::begin_from_pool(pool).await?,
            DbConnection(_, ref mut conn) => TransactionalDatabase::begin_from_conn(conn).await?,
        };
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
    async fn commit(mut self) -> Result<(), InternalError> {
        self.db.commit().await
    }
    async fn rollback(mut self) -> Result<(), InternalError> {
        self.db.rollback().await
    }
}
