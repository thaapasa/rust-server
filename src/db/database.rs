use crate::error::InternalError;
use async_trait::async_trait;
use sqlx::pool::PoolConnection;
use sqlx::postgres::PgRow;
use sqlx::{Execute, Executor, FromRow, PgPool, Pool, Postgres};
use std::sync::Arc;

#[async_trait]
pub trait Database {
    async fn fetch_one<'q, T: for<'r> FromRow<'r, PgRow>>(
        &mut self,
        sql: impl Execute<'q, Postgres> + 'q,
    ) -> Result<T, InternalError>;
}

#[derive(Debug, Clone)]
pub struct DatabasePool(Arc<Pool<Postgres>>);

impl DatabasePool {
    pub async fn init(url: &str) -> Result<Self, InternalError> {
        let pool = PgPool::connect(url).await.map_err(InternalError::from)?;
        Ok(Self(Arc::new(pool)))
    }

    pub async fn acquire(&self) -> Result<DatabaseConnection, InternalError> {
        let conn = self.0.acquire().await.map_err(InternalError::from)?;
        Ok(DatabaseConnection(conn))
    }
}

#[async_trait]
impl Database for DatabasePool {
    async fn fetch_one<'q, T: for<'r> FromRow<'r, PgRow>>(
        &mut self,
        sql: impl Execute<'q, Postgres> + 'q,
    ) -> Result<T, InternalError> {
        let mut conn = self.acquire().await?;
        let row = conn.fetch_one(sql).await?;
        Ok(row)
    }
}

pub struct DatabaseConnection(PoolConnection<Postgres>);

#[async_trait]
impl Database for DatabaseConnection {
    async fn fetch_one<'q, T: for<'r> FromRow<'r, PgRow>>(
        &mut self,
        sql: impl Execute<'q, Postgres> + 'q,
    ) -> Result<T, InternalError> {
        let row = self.0.fetch_one(sql).await?;
        let data = T::from_row(&row)?;
        Ok(data)
    }
}
