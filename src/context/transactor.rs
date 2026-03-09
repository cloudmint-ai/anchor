use crate::*;

#[cfg(feature = "cloud")]
use super::Database;

#[cfg(feature = "cloud")]
use super::Executor;

pub enum Transactor {
    Mocking(MockingTransactor),
    None,
    #[cfg(feature = "cloud")]
    Cloud(CloudTransactor),
}

impl Transactor {
    pub(super) fn mocking() -> Self {
        Self::Mocking(MockingTransactor {
            in_transaction: Arc::new(RwLock::new(false)),
        })
    }

    pub(super) fn none() -> Self {
        Self::None
    }

    #[cfg(feature = "cloud")]
    pub(super) fn cloud(database: Option<Database>) -> Self {
        Self::Cloud(CloudTransactor {
            database: database,
            transaction: Arc::new(RwLock::new(None)),
        })
    }
    // 需要各个实现确保原子且同步的判断是否in transaction
    pub(super) fn clone_out_of_transaction(&self) -> Result<Transactor> {
        match self {
            Self::None => Ok(Self::none()),
            Self::Mocking(transactor) => transactor.clone_out_of_transaction(),
            #[cfg(feature = "cloud")]
            Self::Cloud(transactor) => transactor.clone_out_of_transaction(),
        }
    }

    // 同步判断，仅用于测试框架
    pub(super) fn try_in_transaction(&self) -> Result<bool> {
        match self {
            Self::Mocking(transactor) => transactor.try_in_transaction(),
            _ => Unexpected!("only mocking transactor can try in"),
        }
    }

    // 异步判断，用于更常见的异步业务逻辑，不应用于clone_out_of 的实现内
    pub(super) async fn in_transaction(&self) -> bool {
        match self {
            Self::Mocking(transactor) => transactor.in_transaction().await,
            Self::None => false,
            #[cfg(feature = "cloud")]
            Self::Cloud(transactor) => transactor.in_transaction().await,
        }
    }

    // 需要各个实现原子的确保begin之前并没有在transaction内
    pub(super) async fn begin_transaction(&self) -> Result<()> {
        match self {
            Self::Mocking(transactor) => transactor.begin_transaction().await,
            Self::None => Unexpected!("non transactor can not begin transaction"),
            #[cfg(feature = "cloud")]
            Self::Cloud(transactor) => transactor.begin_transaction().await,
        }
    }

    // 需要各个实现原子的确保commit之前并没有在transaction外
    pub(super) async fn commit_transaction(&self) -> Result<()> {
        match self {
            Self::Mocking(transactor) => transactor.commit_transaction().await,
            Self::None => Unexpected!("non transactor can not commit transaction"),
            #[cfg(feature = "cloud")]
            Self::Cloud(transactor) => transactor.commit_transaction().await,
        }
    }

    #[cfg(feature = "cloud")]
    pub async fn with_executor<'a, F, R>(&'a self, f: F) -> Result<R>
    where
        F: for<'c> FnOnce(
            Executor<'c>,
        ) -> std::pin::Pin<
            Box<dyn std::future::Future<Output = Result<R>> + 'c + Send>,
        >,
        R: 'a,
    {
        match self {
            #[cfg(feature = "cloud")]
            Self::Cloud(transactor) => transactor.with_executor::<'a, F, R>(f).await,
            _ => Unexpected!("with executor not supported"),
        }
    }
}

pub struct MockingTransactor {
    in_transaction: Arc<RwLock<bool>>,
}

impl MockingTransactor {
    fn clone_out_of_transaction(&self) -> Result<Transactor> {
        match self.in_transaction.try_read() {
            Ok(guard) => {
                if !*guard {
                    Ok(Transactor::Mocking(Self {
                        in_transaction: Arc::new(RwLock::new(false)),
                    }))
                } else {
                    Unexpected!("clone in transaction")
                }
            }
            Err(e) => {
                Unexpected!("clone out of transaction fail: {}", e)
            }
        }
    }
    async fn in_transaction(&self) -> bool {
        *self.in_transaction.read().await
    }

    fn try_in_transaction(&self) -> Result<bool> {
        Ok(*self.in_transaction.try_read()?)
    }

    async fn begin_transaction(&self) -> Result<()> {
        let mut guard = self.in_transaction.write().await;
        if *guard {
            Unexpected!("begin in transaction")
        } else {
            *guard = true;
            Ok(())
        }
    }

    async fn commit_transaction(&self) -> Result<()> {
        let mut guard = self.in_transaction.write().await;
        if !*guard {
            Unexpected!("commit out of transaction")
        } else {
            *guard = false;
            Ok(())
        }
    }
}

// TODO 搞一个feature， 支持完全没有sql 的 后端服务
#[cfg(feature = "cloud")]
pub struct CloudTransactor {
    // 可能存在完全没有数据库连接需求的后端服务，因此使用Option
    pub(super) database: Option<Database>,
    pub(super) transaction: Arc<RwLock<Option<sqlx::Transaction<'static, sqlx::Postgres>>>>,
}

#[cfg(feature = "cloud")]
impl CloudTransactor {
    fn clone_out_of_transaction(&self) -> Result<Transactor> {
        match self.transaction.try_read() {
            Ok(guard) => {
                if guard.is_none() {
                    Ok(Transactor::Cloud(Self {
                        database: self.database.clone(),
                        transaction: Arc::new(RwLock::new(None)),
                    }))
                } else {
                    Unexpected!("clone in transaction")
                }
            }
            Err(e) => {
                Unexpected!("clone out of transaction fail: {}", e)
            }
        }
    }
    async fn in_transaction(&self) -> bool {
        self.transaction.read().await.is_some()
    }

    async fn begin_transaction(&self) -> Result<()> {
        let mut guard = self.transaction.write().await;
        if guard.is_some() {
            Unexpected!("begin in transaction")
        } else {
            if let Some(database) = &self.database {
                *guard = Some(database.pool.begin().await?);
                Ok(())
            } else {
                Unexpected!("begin transaction without database")
            }
        }
    }

    async fn commit_transaction(&self) -> Result<()> {
        let mut guard = self.transaction.write().await;
        if let Some(transaction) = guard.take() {
            transaction.commit().await?;
            Ok(())
        } else {
            Unexpected!("commit non-transaction context")
        }
    }

    async fn with_executor<'a, F, R>(&'a self, f: F) -> Result<R>
    where
        F: for<'c> FnOnce(
            Executor<'c>,
        ) -> std::pin::Pin<
            Box<dyn std::future::Future<Output = Result<R>> + 'c + Send>,
        >,
        R: 'a,
    {
        let mut guard = self.transaction.write().await;
        let result = match guard.as_mut() {
            Some(transaction) => {
                let executor = Executor::Transaction(transaction);
                f(executor).await
            }
            None => {
                if let Some(database) = &self.database {
                    f(Executor::Pool(database.pool.clone())).await
                } else {
                    Unexpected!("try run executor without database")
                }
            }
        };
        result
    }
}
