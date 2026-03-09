use crate::*;

#[cfg(feature = "cloud")]
mod database;
#[cfg(feature = "cloud")]
pub use database::*;

#[cfg(feature = "cloud")]
mod executor;
#[cfg(feature = "cloud")]
use executor::*;

// TODO simplify async feature
mod transactor;
use transactor::*;

pub struct Context {
    pub trace_id: Id,
    pub transactor: Transactor,
}

// 注意只在 非 in transaction 的时候 transactor 才允许 clone, 否则panic
// clone 之后各自 begin 应该是各自的transaction
impl Clone for Context {
    fn clone(&self) -> Self {
        let transactor = match self.transactor.clone_out_of_transaction() {
            Ok(transactor) => transactor,
            Err(e) => {
                panic!("transaction colne out of transaction fail: {}", e)
            }
        };
        Self {
            trace_id: self.trace_id.clone(),
            transactor,
        }
    }
}

impl Context {
    pub const BACKGROUND_TRACE_ID: Id = Id::ZERO;

    // TODO 确保只在测试中使用 且名字不那么突兀
    pub fn mocking() -> Self {
        Self::new(Self::BACKGROUND_TRACE_ID, Transactor::mocking())
    }

    fn new(trace_id: Id, transactor: Transactor) -> Self {
        Self {
            trace_id,
            transactor,
        }
    }

    #[cfg(feature = "cloud")]
    // 仅用于后台进程
    pub fn background(database: Option<Database>) -> Self {
        Self::cloud(Self::BACKGROUND_TRACE_ID, database)
    }

    #[cfg(not(feature = "cloud"))]
    // 仅用于后台进程
    pub fn background() -> Self {
        Self::new(Self::BACKGROUND_TRACE_ID, Transactor::none())
    }

    #[cfg(feature = "cloud")]
    pub fn cloud(trace_id: Id, database: Option<Database>) -> Self {
        Self::new(trace_id, Transactor::cloud(database))
    }

    // 仅由 transaction 宏调用
    pub async fn _begin_transaction(&self) -> Result<()> {
        self.transactor.begin_transaction().await
    }

    // 仅由 transaction 宏调用
    pub async fn _commit_transaction(&self) -> Result<()> {
        self.transactor.commit_transaction().await
    }

    pub async fn in_transaction(&self) -> bool {
        self.transactor.in_transaction().await
    }

    // 仅用于测试框架
    pub(crate) fn try_in_transaction(&self) -> Result<bool> {
        self.transactor.try_in_transaction()
    }
}

// 预期事务内应只有数据库操作和可重复无状态的纯计算，故错误都可以自由回滚
// 不回滚的操作应该进行错误处理和闭包变量控制
#[macro_export]
macro_rules! transaction {
    ($context:ident, {$($body:tt)*}) => {
        {
            $context._begin_transaction().await?;
            let result = { $($body)* };
            $context._commit_transaction().await?;
            result
        }
    };
}

#[macro_export]
macro_rules! execute {
    ($context:ident, $query:expr, $($args:tt)*) => {
        $context.transactor.with_executor(|executor| {
            Box::pin(async move {
                sqlx::query!($query, $($args)*)
                    .execute(executor).await?
                })
        })
    };
}

#[macro_export]
macro_rules! fetch_scalar {
    ($context:ident, $query:expr, $($args:tt)*) => {
        $context.transactor.with_executor(|executor| {
            Box::pin(async move {
                // TODO rec unwrap
                let scalar = sqlx::query_scalar!($query, $($args)*).fetch_one(executor).await?;
                Ok(scalar)
            })
        })
    };
}

#[macro_export]
macro_rules! fetch_option_scalar {
    ($context:ident, $query:expr, $($args:tt)*) => {
        $context.transactor.with_executor(|executor| {
            Box::pin(async move {
                sqlx::query_scalar!($query, $($args)*).fetch_optional(executor).await?
            })
        })
    };
}

#[macro_export]
macro_rules! fetch_one {
    ($context:ident,$type:ty, $query:expr, $($args:tt)*) => {
        $context.transactor.with_executor(|executor| {
            Box::pin(async move {
                sqlx::query_as!($type, $query, $($args)*).fetch_one(executor).await?
            })
        })
    };
}

#[macro_export]
macro_rules! fetch_option {
    ($context:ident,$type:ty, $query:expr, $($args:tt)*) => {
        $context.transactor.with_executor(|executor| {
            Box::pin(async move {
                sqlx::query_as!($type, $query, $($args)*).fetch_optional(executor).await?
            })
        })
    };
}

#[macro_export]
macro_rules! fetch_all {
    ($context:ident, $type:ty, $query:expr, $($args:tt)*) => {
        $context.transactor.with_executor(|executor| {
            Box::pin(async move {
                sqlx::query_as!($type, $query, $($args)*).fetch_all(executor).await?
            })
        })
    };
}

// TODO 编译期间检查 query 是否结尾是 for update
// TODO 搞一个save 函数 编译期间检查是否是 for update
// 使用 SELECT pg_advisory_xact_lock(1) 用于verison = 0
// 当version 不匹配的时候，不触发行锁
#[macro_export]
macro_rules! lock_version {
    ($context:ident, $select_version_where_id_and_version_for_update_query:expr, $id:expr, $version:expr) => {
        let version: Option<i64> =
            sqlx::query_option_scalar!($select_version_for_update_query, $id)
                .fetch_one(&mut *transaction)
                .await?;

        match current_version {
            Some(version) => {
                // 此处如果 sql 正确书写，应该肯定是匹配的。因为version 不匹配应该是 None
                if version != $version {
                    return Unexpected!("verison not match: {} != {}", version, $version)
                }
            }
            None => {
                if $version == 0 {
                    // 先加咨询锁，避免 version 写锁争夺，方便后续事务失败时的 version 回滚
                    sqlx::query!("SELECT pg_advisory_xact_lock($1);", $id).execute(&mut *transaction).await?;
                } else {
                    return Unexpected!("version not match: {} != 0", $version);
                }
            }
        }
    }
}

// TODO 让anchor 可以独立自测
// #[cfg(feature = "cloud")]
// tests! {
//     #[ignore]
//     async fn test_macro_gramma() {
//         let context = Context::cloud(Id::from(10), None);
//         transaction!(context, {
//             execute!(context, "select {};", 1);
//             fetch_scalar!(context, "select {};", 1);
//         })

//         // let
//         // fetch_one!()
//     }
// }
