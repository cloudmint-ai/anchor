use crate::*;
use sqlx::pool::Pool;
use sqlx::postgres::{PgPoolOptions, Postgres};

#[derive(Clone)]
pub struct Database {
    pub _pool: Pool<Postgres>,
}

impl Database {
    // 配合同步依赖注入，使用同步方法进行初始化
    pub fn new(url: String, max_connections: u32) -> Result<Self> {
        let runtime = Runtime::new()?;
        let pool = runtime.block_on(async {
            PgPoolOptions::new()
                .max_connections(max_connections)
                .connect(&url)
                .await
        })?;
        Ok(Self { _pool: pool })
    }
    pub async fn health(&self) -> Result<()> {
        let one: i64 = sqlx::query_scalar("select 1")
            .fetch_one(&self._pool)
            .await?;
        if one != 1 {
            return Unexpected!("select 1 result: {}", one);
        }
        Ok(())
    }
}

#[macro_export]
macro_rules! execute {
    ($database:expr, $context:ident, $query:expr, $($args:tt)*) => {
        if let Some(mut tx) = $context.transaction.lock().await.as_mut() {
            sqlx::query!($query, $($args)*)
                .execute(&mut **tx)
                .await?;
            Ok(())
        } else {
            return Unexpected!("execute without lock in transaction");
        }
    };
}

// 使用 SELECT pg_advisory_xact_lock(1)
// 用咨询锁保证 version 写锁之前就已经完成排他，这样version 的写锁就不需要长期传递和维护。
// 由于咨询锁的存在，不再需要 select for update 的行锁, 但仍然需要用 version 自增持久化来保证并发控制
#[macro_export]
macro_rules! lock {
    ($database:expr, $context:ident, $select_version_query:expr, $update_version_query:expr, $insert_version_query:expr, $id:expr, $version:expr, $($args:tt)*) => {
        if $context.in_transaction().await? {
            Unexpected!("transaction activated before lock")
        } else {
            let mut transaction = $database._pool.begin().await?;
            // 先加咨询锁，避免 version 写锁争夺，方便后续事务失败时的 version 回滚
            sqlx::query!("SELECT pg_advisory_xact_lock($1);", $id).execute(&mut *transaction).await?;

            if $version.is_zero() {
                // 由于有前置锁，不应当存在冲突，故不await
                $version._increase()?;
                sqlx::query!($insert_version_query, $id, $version, $($args)*)
                    .execute(&mut *transaction)
                    .await?;
            } else {
                // 由于有前置锁，不应当存在冲突，故不await
                $version._increase()?;
                let current_version: i64 =
                    sqlx::query_scalar!($select_version_query, $id)
                        .fetch_one(&mut *transaction)
                        .await?;

                if current_version + 1 != i64::from(&$version) {
                    return Unexpected!("version not match {} + 1 != {}", current_version, $version);
                } else {
                    sqlx::query!($update_version_query, $id, $version)
                        .execute(&mut *transaction)
                        .await?;

                }
            }
            $context.bind_transaction(transaction, &$version).await
        }
    };

}

#[macro_export]
macro_rules! fetch_scalar {
    ($database:expr, $context:ident, $query:expr, $($args:tt)*) => {
        if let Some(mut tx) = $context.transaction.lock().await.as_mut() {
            sqlx::query_scalar!($query, $($args)*).fetch_one(&mut **tx).await?
        } else {
            // 后续可以在这个条件下 做 query 缓存化
            sqlx::query_scalar!($query, $($args)*).fetch_one(&$database._pool).await?
        }
    };
}

#[macro_export]
macro_rules! fetch_one {
    ($database:expr, $context:ident,$type:ty, $query:expr, $($args:tt)*) => {
        if let Some(mut tx) = $context.transaction.lock().await.as_mut() {
            sqlx::query_as!($type, $query, $($args)*).fetch_one(&mut **tx).await?
        } else {
            // 后续可以在这个条件下 做 query 缓存化
            sqlx::query_as!($type, $query, $($args)*).fetch_one(&$database._pool).await?
        }
    };
}

#[macro_export]
macro_rules! fetch_option {
    ($database:expr, $context:ident,$type:ty, $query:expr, $($args:tt)*) => {
        if let Some(mut tx) = $context.transaction.lock().await.as_mut() {
            sqlx::query_as!($type, $query, $($args)*).fetch_optional(&mut **tx).await?
        } else {
            // 后续可以在这个条件下 做 query 缓存化
            sqlx::query_as!($type, $query, $($args)*).fetch_optional(&$database._pool).await?
        }
    };
}

#[macro_export]
macro_rules! fetch_all {
    ($database:expr, $context:ident, $type:ty, $query:expr, $($args:tt)*) => {
        if let Some(tx) = $context.transaction.lock().await.as_mut() {
            sqlx::query_as!($type, $query, $($args)*).fetch_all(&mut **tx).await?
        } else {
            sqlx::query_as!($type, $query, $($args)*).fetch_all(&$database._pool).await?
        }
    };
}
