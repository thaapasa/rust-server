use futures_core::future::BoxFuture;
use futures_util::FutureExt;
use sqlx::{
    Acquire, Execute, Executor, FromRow, PgConnection, PgPool, PgTransaction, Pool, Postgres,
};
use sqlx::pool::PoolConnection;
use sqlx::postgres::{PgQueryResult, PgRow};

use crate::error::InternalError;

#[derive(Debug)]
pub enum Database {
    DbPool(Pool<Postgres>),
    DbConnection(Pool<Postgres>, PoolConnection<Postgres>),
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
        Ok(Self::DbPool(pool))
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
}

pub trait DatabaseAccess {
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
}

impl DatabaseAccess for Database {
    fn execute<'e, 'q: 'e, E>(
        &'e mut self,
        query: E,
    ) -> BoxFuture<'e, Result<PgQueryResult, InternalError>>
    where
        E: 'q + Execute<'q, Postgres>,
    {
        match self {
            Self::DbPool(pool) => Box::pin(async move {
                let mut conn = pool.acquire().await.map_err(InternalError::from)?;
                conn.execute(query).await.map_err(InternalError::from)
            })
            .boxed(),
            Self::DbConnection(_, ref mut conn) => {
                Box::pin(async move { conn.execute(query).await.map_err(InternalError::from) })
            }
        }
    }

    fn fetch_rows<'e, 'q: 'e, E>(
        &'e mut self,
        query: E,
    ) -> BoxFuture<'e, Result<Vec<PgRow>, InternalError>>
    where
        E: 'q + Execute<'q, Postgres>,
    {
        match self {
            Self::DbPool(pool) => Box::pin(async move {
                let mut conn = pool.acquire().await.map_err(InternalError::from)?;
                conn.fetch_all(query).await.map_err(InternalError::from)
            }),
            Self::DbConnection(_, conn) => {
                Box::pin(async move { conn.fetch_all(query).await.map_err(InternalError::from) })
            }
        }
    }
}

pub struct TransactionalDatabase<'a> {
    finished: bool,
    connection: PgTransaction<'a>,
}

impl DatabaseAccess for TransactionalDatabase<'_> {
    fn execute<'e, 'q: 'e, E>(
        &'e mut self,
        query: E,
    ) -> BoxFuture<'e, Result<PgQueryResult, InternalError>>
    where
        E: 'q + Execute<'q, Postgres>,
    {
        Box::pin(async move {
            self.connection
                .execute(query)
                .await
                .map_err(InternalError::from)
        })
    }

    fn fetch_rows<'e, 'q: 'e, E>(
        &'e mut self,
        query: E,
    ) -> BoxFuture<'e, Result<Vec<PgRow>, InternalError>>
    where
        E: 'q + Execute<'q, Postgres>,
    {
        Box::pin(async move {
            self.connection
                .fetch_all(query)
                .await
                .map_err(InternalError::from)
        })
    }
}

impl TransactionalDatabase<'_> {
    pub async fn begin_from_pool(pool: &Pool<Postgres>) -> Result<Self, InternalError> {
        let tx = pool.begin().await.map_err(InternalError::from)?;
        Ok(TransactionalDatabase {
            finished: false,
            connection: tx,
        })
    }

    pub async fn begin_from_conn(
        conn: &mut PgConnection,
    ) -> Result<TransactionalDatabase, InternalError> {
        let tx = conn.begin().await.map_err(InternalError::from)?;
        Ok(TransactionalDatabase {
            finished: false,
            connection: tx,
        })
    }

    pub async fn begin(&mut self) -> Result<TransactionalDatabase, InternalError> {
        let tx = self.connection.begin().await.map_err(InternalError::from)?;
        Ok(TransactionalDatabase {
            finished: false,
            connection: tx,
        })
    }

    pub async fn rollback(mut self) -> Result<(), InternalError> {
        if self.finished {
            return Err(InternalError::message(
                "Transaction already finished".to_string(),
            ));
        }
        self.connection
            .rollback()
            .await
            .map_err(InternalError::from)?;
        self.finished = true;
        Ok(())
    }

    pub async fn commit(mut self) -> Result<(), InternalError> {
        if self.finished {
            return Err(InternalError::message(
                "Transaction already finished".to_string(),
            ));
        }
        self.connection
            .commit()
            .await
            .map_err(InternalError::from)?;
        self.finished = true;
        Ok(())
    }
}

pub trait DatabaseAccessExt {
    async fn fetch_all<'q, T: for<'r> FromRow<'r, PgRow>>(
        &mut self,
        query: impl Execute<'q, Postgres> + 'q,
    ) -> Result<Vec<T>, InternalError>;

    async fn fetch_one<'q, T: for<'r> FromRow<'r, PgRow>>(
        &mut self,
        query: impl Execute<'q, Postgres> + 'q,
    ) -> Result<T, InternalError>;

    async fn fetch_optional<'q, T: for<'r> FromRow<'r, PgRow>>(
        &mut self,
        query: impl Execute<'q, Postgres> + 'q,
    ) -> Result<Option<T>, InternalError>;
}

impl<D: DatabaseAccess> DatabaseAccessExt for D {
    async fn fetch_all<'q, T: for<'r> FromRow<'r, PgRow>>(
        &mut self,
        query: impl Execute<'q, Postgres> + 'q,
    ) -> Result<Vec<T>, InternalError> {
        self.fetch_rows(query)
            .await?
            .into_iter()
            .map(|row| FromRow::from_row(&row).map_err(InternalError::from))
            .collect()
    }

    async fn fetch_one<'q, T: for<'r> FromRow<'r, PgRow>>(
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

    async fn fetch_optional<'q, T: for<'r> FromRow<'r, PgRow>>(
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
}
