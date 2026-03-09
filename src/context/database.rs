use crate::*;
use sqlx::pool::Pool;
use sqlx::postgres::{PgPoolOptions, Postgres};

#[derive(Debug, Clone)]
pub struct Database {
    pub pool: Pool<Postgres>,
}

impl Database {
    pub async fn new(url: String, max_connections: u32) -> Result<Self> {
        let pool = PgPoolOptions::new()
            .max_connections(max_connections)
            .connect(&url)
            .await?;
        let result = Self { pool };
        result.health().await?;
        Ok(result)
    }
    pub async fn health(&self) -> Result<()> {
        let one: i64 = sqlx::query_scalar("select 1").fetch_one(&self.pool).await?;
        if one != 1 {
            return Unexpected!("select 1 result: {}", one);
        }
        Ok(())
    }
}
