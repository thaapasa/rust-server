use crate::error::InternalError;
use macros::sql;
use sqlx::pool::PoolConnection;
use sqlx::postgres::PgRow;
use sqlx::{Execute, Executor, FromRow, PgPool, Pool, Postgres};
use std::ops::{Deref, DerefMut};
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
        if let Some(savepoint) = savepoint.clone() {
            db.execute(sql!("SAVEPOINT {savepoint:raw}")).await?;
        } else {
            db.execute(sql!("BEGIN")).await?;
        }
        Ok(Self { db, savepoint })
    }

    pub async fn rollback(self) -> Result<(), InternalError> {
        if let Some(savepoint) = &self.savepoint {
            self.db
                .execute(sql!("ROLLBACK TO SAVEPOINT {savepoint:raw}"))
                .await
        } else {
            self.db.execute(sql!("ROLLBACK")).await
        }
    }

    pub async fn commit(self) -> Result<(), InternalError> {
        if let Some(savepoint) = &self.savepoint {
            self.db
                .execute(sql!("RELEASE SAVEPOINT {savepoint:raw}"))
                .await
        } else {
            self.db.execute(sql!("COMMIT")).await
        }
    }
}
