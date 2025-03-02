use std::ops::{Deref, DerefMut};

use futures_core::future::BoxFuture;
use futures_util::FutureExt;
use sqlx::{Acquire, Execute, Executor, FromRow, PgPool, PgTransaction, Pool, Postgres};
use sqlx::pool::PoolConnection;
use sqlx::postgres::{PgQueryResult, PgRow};

use crate::error::InternalError;

pub trait DatabaseAccess: Send + Sync {
    fn execute<'e, 'q: 'e, E>(
        &'e mut self,
        query: E,
    ) -> BoxFuture<'e, Result<PgQueryResult, InternalError>>
    where
        E: 'q + Execute<'q, Postgres>;

    fn fetch_rows<'e, 'q: 'e, E>(
        &'e mut self,
        query: E,
    ) -> BoxFuture<'e, Result<Vec<PgRow>, InternalError>>
    where
        E: 'q + Execute<'q, Postgres>;

    fn fetch_all<'e, 'q: 'e, T: for<'r> FromRow<'r, PgRow>>(
        &'e mut self,
        query: impl Execute<'q, Postgres> + 'q,
    ) -> BoxFuture<'e, Result<Vec<T>, InternalError>> {
        Box::pin(async move {
            self.fetch_rows(query)
                .await?
                .into_iter()
                .map(|row| FromRow::from_row(&row).map_err(InternalError::from))
                .collect()
        })
    }

    fn fetch_one<'e, 'q: 'e, T: for<'r> FromRow<'r, PgRow>>(
        &'e mut self,
        query: impl Execute<'q, Postgres> + 'q,
    ) -> BoxFuture<'e, Result<T, InternalError>> {
        Box::pin(async move {
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
        })
    }

    fn fetch_optional<'e, 'q: 'e, T: for<'r> FromRow<'r, PgRow>>(
        &'e mut self,
        query: impl Execute<'q, Postgres> + 'q,
    ) -> BoxFuture<'e, Result<Option<T>, InternalError>> {
        Box::pin(async move {
            let mut rows = self.fetch_all(query).await?;
            if rows.len() > 1 {
                return Err(InternalError::message(format!(
                    "Too many results, expected one or zero results, received {}",
                    rows.len()
                )));
            }
            Ok(rows.pop())
        })
    }
}

#[derive(Debug, Clone)]
pub struct DatabasePool(PgPool);

impl Deref for DatabasePool {
    type Target = PgPool;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DatabasePool {
    pub async fn init_pool(url: &str) -> Result<Self, InternalError> {
        let pool = PgPool::connect(url).await.map_err(InternalError::from)?;
        Ok(DatabasePool(pool))
    }
}

impl DatabaseAccess for DatabasePool {
    fn execute<'e, 'q: 'e, E: 'q + Execute<'q, Postgres>>(
        &'e mut self,
        query: E,
    ) -> BoxFuture<'e, Result<PgQueryResult, InternalError>> {
        Box::pin(async move {
            let mut conn = self.0.acquire().await.map_err(InternalError::from)?;
            conn.execute(query).await.map_err(InternalError::from)
        })
        .boxed()
    }

    fn fetch_rows<'e, 'q: 'e, E: 'q + Execute<'q, Postgres>>(
        &'e mut self,
        query: E,
    ) -> BoxFuture<'e, Result<Vec<PgRow>, InternalError>> {
        Box::pin(async move {
            let mut conn = self.0.acquire().await.map_err(InternalError::from)?;
            conn.fetch_all(query).await.map_err(InternalError::from)
        })
    }
}

#[derive(Debug)]
pub struct DatabaseConnection(PoolConnection<Postgres>);

impl Deref for DatabaseConnection {
    type Target = PoolConnection<Postgres>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for DatabaseConnection {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl DatabaseAccess for DatabaseConnection {
    fn execute<'e, 'q: 'e, E: 'q + Execute<'q, Postgres>>(
        &'e mut self,
        query: E,
    ) -> BoxFuture<'e, Result<PgQueryResult, InternalError>> {
        Box::pin(async move { self.0.execute(query).await.map_err(InternalError::from) })
    }

    fn fetch_rows<'e, 'q: 'e, E: 'q + Execute<'q, Postgres>>(
        &'e mut self,
        query: E,
    ) -> BoxFuture<'e, Result<Vec<PgRow>, InternalError>> {
        Box::pin(async move { self.0.fetch_all(query).await.map_err(InternalError::from) })
    }
}

pub struct TransactionalConnection<'a> {
    finished: bool,
    tx: PgTransaction<'a>,
}

impl DatabaseAccess for TransactionalConnection<'_> {
    fn execute<'e, 'q: 'e, E: 'q + Execute<'q, Postgres>>(
        &'e mut self,
        query: E,
    ) -> BoxFuture<'e, Result<PgQueryResult, InternalError>> {
        Box::pin(async move { self.tx.execute(query).await.map_err(InternalError::from) })
    }

    fn fetch_rows<'e, 'q: 'e, E: 'q + Execute<'q, Postgres>>(
        &'e mut self,
        query: E,
    ) -> BoxFuture<'e, Result<Vec<PgRow>, InternalError>> {
        Box::pin(async move { self.tx.fetch_all(query).await.map_err(InternalError::from) })
    }
}

impl TransactionalConnection<'_> {
    pub async fn begin_from_pool(pool: &Pool<Postgres>) -> Result<Self, InternalError> {
        let tx = pool.begin().await.map_err(InternalError::from)?;
        Ok(TransactionalConnection {
            finished: false,
            tx,
        })
    }

    pub async fn begin(&mut self) -> Result<TransactionalConnection, InternalError> {
        let tx = self.tx.begin().await.map_err(InternalError::from)?;
        Ok(TransactionalConnection {
            finished: false,
            tx,
        })
    }

    pub async fn rollback(mut self) -> Result<(), InternalError> {
        if self.finished {
            return Err(InternalError::message(
                "Transaction already finished".to_string(),
            ));
        }
        self.tx.rollback().await.map_err(InternalError::from)?;
        self.finished = true;
        Ok(())
    }

    pub async fn commit(mut self) -> Result<(), InternalError> {
        if self.finished {
            return Err(InternalError::message(
                "Transaction already finished".to_string(),
            ));
        }
        self.tx.commit().await.map_err(InternalError::from)?;
        self.finished = true;
        Ok(())
    }
}
