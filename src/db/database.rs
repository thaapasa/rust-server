use crate::error::InternalError;
use sql::sql;
use sqlx::pool::PoolConnection;
use sqlx::postgres::{PgRow, PgTransactionManager};
use sqlx::{Execute, Executor, FromRow, PgPool, Pool, Postgres, TransactionManager};
use std::ops::{Deref, DerefMut};
use std::sync::Arc;
use tracing::{debug, warn};

#[derive(Debug)]
pub enum Database {
    DbPool(Arc<Pool<Postgres>>),
    DbConnection(Arc<Pool<Postgres>>, PoolConnection<Postgres>),
}

impl Clone for Database {
    fn clone(&self) -> Self {
        match self {
            Self::DbPool(pool) => Self::DbPool(pool.clone()),
            _ => panic!("Can't clone DB connections"),
        }
    }
}

impl Database {
    pub async fn init_pool(url: &str) -> Result<Self, InternalError> {
        let pool = PgPool::connect(url).await.map_err(InternalError::from)?;
        Ok(Self::DbPool(Arc::new(pool)))
    }

    pub async fn fetch_rows<'q>(
        &mut self,
        query: impl Execute<'q, Postgres> + 'q,
    ) -> Result<Vec<PgRow>, InternalError> {
        match self {
            Self::DbPool(pool) => {
                let mut conn = pool.acquire().await.map_err(InternalError::from)?;
                conn.fetch_all(query).await.map_err(InternalError::from)
            }
            Self::DbConnection(_, conn) => conn.fetch_all(query).await.map_err(InternalError::from),
        }
    }

    pub async fn connection(&self) -> Result<Self, InternalError> {
        let pool = match self {
            Self::DbPool(pool) => pool,
            Self::DbConnection(pool, _) => pool,
        };
        Ok(Self::DbConnection(
            pool.clone(),
            pool.acquire().await.map_err(InternalError::from)?,
        ))
    }

    pub fn connection_mut(&mut self) -> Option<&mut PoolConnection<Postgres>> {
        match self {
            Self::DbPool(..) => None,
            Self::DbConnection(_, conn) => Some(conn),
        }
    }

    pub async fn fetch_all<'q, T: for<'r> FromRow<'r, PgRow>>(
        &mut self,
        query: impl Execute<'q, Postgres> + 'q,
    ) -> Result<Vec<T>, InternalError> {
        self.fetch_rows(query)
            .await?
            .into_iter()
            .map(|row| FromRow::from_row(&row).map_err(InternalError::from))
            .collect()
    }

    pub async fn fetch_one<'q, T: for<'r> FromRow<'r, PgRow>>(
        &mut self,
        query: impl Execute<'q, Postgres> + 'q,
    ) -> Result<T, InternalError> {
        let mut rows = self.fetch_all(query).await?;
        if rows.len() > 1 {
            return Err(InternalError::message(format!(
                "Too many results, expected exactly one result, received {}",
                rows.len()
            )));
        }
        rows.pop().ok_or(InternalError::message(
            "No results for query, expected exactly one result".to_string(),
        ))
    }

    pub async fn fetch_optional<'q, T: for<'r> FromRow<'r, PgRow>>(
        &mut self,
        query: impl Execute<'q, Postgres> + 'q,
    ) -> Result<Option<T>, InternalError> {
        let mut rows = self.fetch_all(query).await?;
        if rows.len() > 1 {
            return Err(InternalError::message(format!(
                "Too many results, expected one or zero results, received {}",
                rows.len()
            )));
        }
        Ok(rows.pop())
    }

    pub async fn execute<'q>(
        &mut self,
        query: impl Execute<'q, Postgres> + 'q,
    ) -> Result<(), InternalError> {
        match self {
            Self::DbPool(pool) => {
                let mut conn = pool.acquire().await.map_err(InternalError::from)?;
                conn.execute(query).await.map_err(InternalError::from)?;
            }
            Self::DbConnection(_, conn) => {
                conn.execute(query).await.map_err(InternalError::from)?;
            }
        }
        Ok(())
    }
}

pub struct TransactionalDatabase<'a> {
    db: &'a mut Database,
    savepoint: Option<String>,
    next_savepoint: usize,
    finished: bool,
}

impl Deref for TransactionalDatabase<'_> {
    type Target = Database;

    fn deref(&self) -> &Self::Target {
        self.db
    }
}

impl DerefMut for TransactionalDatabase<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.db
    }
}

impl<'a> TransactionalDatabase<'a> {
    pub async fn begin(
        db: &'a mut Database,
        savepoint: Option<String>,
    ) -> Result<Self, InternalError> {
        if let Database::DbPool(..) = db {
            panic!("Can't begin transaction directly on DB pool");
        }
        if let Some(savepoint) = savepoint.clone() {
            db.execute(sql!("SAVEPOINT {savepoint:id}")).await?;
        } else {
            db.execute(sql!("BEGIN")).await?;
        }
        Ok(Self {
            db,
            savepoint,
            next_savepoint: 1,
            finished: false,
        })
    }

    pub fn nested_savepoint_id(&mut self) -> String {
        let res = if let Some(savepoint) = &self.savepoint {
            format!("{savepoint}.{}", self.next_savepoint)
        } else {
            format!("savepoint.{}", self.next_savepoint)
        };
        self.next_savepoint += 1;
        res
    }

    pub async fn rollback(mut self) -> Result<(), InternalError> {
        if self.finished {
            return Err(InternalError::message(
                "Transaction already finished".to_string(),
            ));
        }
        if let Some(savepoint) = &self.savepoint {
            self.db
                .execute(sql!("ROLLBACK TO SAVEPOINT {savepoint:id}"))
                .await?;
        } else {
            self.db.execute(sql!("ROLLBACK")).await?;
        }
        self.finished = true;
        Ok(())
    }

    pub async fn commit(mut self) -> Result<(), InternalError> {
        if self.finished {
            return Err(InternalError::message(
                "Transaction already finished".to_string(),
            ));
        }
        if let Some(savepoint) = &self.savepoint {
            self.db
                .execute(sql!("RELEASE SAVEPOINT {savepoint:id}"))
                .await?;
        } else {
            self.db.execute(sql!("COMMIT")).await?;
        }
        self.finished = true;
        Ok(())
    }
}

impl Drop for TransactionalDatabase<'_> {
    fn drop(&mut self) {
        debug!("Dropping transaction");
        if !self.finished {
            warn!("Unfinished transaction, starting rollback!");
            PgTransactionManager::start_rollback(
                self.db
                    .connection_mut()
                    .expect("Transaction should always have a connection"),
            )
        }
    }
}
