use crate::error::InternalError;
use sqlx::pool::PoolConnection;
use sqlx::postgres::PgRow;
use sqlx::{Execute, Executor, FromRow, PgPool, Pool, Postgres};
use std::sync::Arc;

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
}
