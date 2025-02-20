use crate::error::InternalError;
use async_trait::async_trait;
use sqlx::pool::PoolConnection;
use sqlx::postgres::PgRow;
use sqlx::{Execute, Executor, FromRow, PgPool, Pool, Postgres};
use std::sync::Arc;

#[async_trait]
pub trait Database {
    async fn fetch_all<'q, T: for<'r> FromRow<'r, PgRow>>(
        &mut self,
        query: impl Execute<'q, Postgres> + 'q,
    ) -> Result<Vec<T>, InternalError>;

    async fn fetch_one<'q, T: for<'r> FromRow<'r, PgRow>>(
        &mut self,
        query: impl Execute<'q, Postgres> + 'q,
    ) -> Result<T, InternalError> {
        let mut rows = self.fetch_all(query).await?;
        if rows.len() > 1 {
            return Err(InternalError::message(format!(
                "Too many result, expected exactly one result, received {}",
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

    async fn execute<'q>(
        &mut self,
        query: impl Execute<'q, Postgres> + 'q,
    ) -> Result<(), InternalError>;
}

#[derive(Debug, Clone)]
pub struct DatabasePool(Arc<Pool<Postgres>>);

impl DatabasePool {
    pub async fn init(url: &str) -> Result<Self, InternalError> {
        let pool = PgPool::connect(url).await.map_err(InternalError::from)?;
        Ok(Self(Arc::new(pool)))
    }

    pub fn db_pool(&self) -> &PgPool {
        &self.0
    }

    pub async fn acquire(&self) -> Result<DatabaseConnection, InternalError> {
        let conn = self.0.acquire().await.map_err(InternalError::from)?;
        Ok(DatabaseConnection(conn))
    }
}

#[async_trait]
impl Database for DatabasePool {
    async fn fetch_all<'q, T: for<'r> FromRow<'r, PgRow>>(
        &mut self,
        query: impl Execute<'q, Postgres> + 'q,
    ) -> Result<Vec<T>, InternalError> {
        let mut conn = self.acquire().await?;
        let row = conn.fetch_all(query).await?;
        Ok(row)
    }

    async fn execute<'q>(
        &mut self,
        query: impl Execute<'q, Postgres> + 'q,
    ) -> Result<(), InternalError> {
        let mut conn = self.acquire().await?;
        conn.execute(query).await
    }
}

pub struct DatabaseConnection(PoolConnection<Postgres>);

#[async_trait]
impl Database for DatabaseConnection {
    async fn fetch_all<'q, T: for<'r> FromRow<'r, PgRow>>(
        &mut self,
        query: impl Execute<'q, Postgres> + 'q,
    ) -> Result<Vec<T>, InternalError> {
        let rows = self.0.fetch_all(query).await?;
        rows.into_iter()
            .map(|row| T::from_row(&row))
            .collect::<Result<_, _>>()
            .map_err(InternalError::from)
    }

    async fn execute<'q>(
        &mut self,
        query: impl Execute<'q, Postgres> + 'q,
    ) -> Result<(), InternalError> {
        self.0.execute(query).await?;
        Ok(())
    }
}
