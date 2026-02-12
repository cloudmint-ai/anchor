use crate::*;
#[cfg(feature = "cloud")]
use sqlx::{Postgres, Transaction};
#[cfg(not(feature = "runtime"))]
use std::sync::Mutex;

pub struct Context {
    pub trace_id: Id,

    mocking: bool,
    // 使用 Arc<Mutex> 避免 &mut 传递
    mocking_transaction: Arc<Mutex<bool>>,

    // 注意，为了使得 clone 之后的 None 能够独立的开启事务，
    // transaction 在 clone 之后，不是复制 而是新建一个 Arc<Mutex>
    // 使用 Arc<Mutex> 是为了 Context 不需要 &mut 传递
    #[cfg(feature = "cloud")]
    pub transaction: Arc<Mutex<Option<Transaction<'static, Postgres>>>>,

    // 用于乐观锁维护版本提交和回滚，注意是数据库事务开启，获得数据库锁之后，再获得读写锁进行自增，rollback则自减
    // 在 clone 之后，同样是新建一个 Arc<Mutex>
    version_for_transaction: Arc<Mutex<Option<Version>>>,
}

// 此处clone 是用于业务代码
impl Clone for Context {
    fn clone(&self) -> Self {
        if self.mocking {
            #[cfg(feature = "runtime")]
            match self.mocking_transaction.try_lock() {
                Ok(mocking_transaction) => {
                    if *mocking_transaction {
                        // 阻止 transaction 进行clone
                        panic!("DO NOT clone Context in mocking_transaction")
                    }
                }
                Err(e) => panic!("DO NOT clone Context in mocking_transaction: {}", e),
            }
        }

        #[cfg(feature = "cloud")]
        match self.transaction.try_lock() {
            Ok(transaction) => {
                if transaction.is_some() {
                    // 阻止 transaction 进行clone
                    panic!("DO NOT clone Context in transaction")
                }
            }
            Err(e) => panic!("DO NOT clone Context in transaction: {}", e),
        }

        match self.version_for_transaction.try_lock() {
            Ok(version) => {
                if version.is_some() {
                    panic!("DO NOT clone Context with version_for_transaction")
                }
            }
            Err(e) => panic!("DO NOT clone Context with version_for_transaction: {}", e),
        }

        Self {
            trace_id: self.trace_id.clone(),
            mocking: self.mocking,
            // 注意，clone 后不是 复制 Mutex，而是新建 Mutex, 保证clone后的Context 能够独立创建事务
            mocking_transaction: Arc::new(Mutex::new(false)),

            #[cfg(feature = "cloud")]
            // 注意，clone 后不是 复制 Mutex，而是新建 Mutex, 保证clone后的Context 能够独立创建事务
            transaction: Arc::new(Mutex::new(None)),

            // 注意，clone 后不是 复制 Mutex，而是新建 Mutex, 保证clone后的Context 能够独立创建事务
            version_for_transaction: Arc::new(Mutex::new(None))
        }
    }
}

impl Context {
    pub const BACKGROUND_TRACE_ID: Id = Id::ZERO;

    // 仅用于后台进程
    pub fn background() -> Self {
        Self::new(Self::BACKGROUND_TRACE_ID)
    }

    // TODO 确保只在测试中使用 且名字不那么突兀
    pub fn mocking() -> Self {
        let mut result = Self::new(Self::BACKGROUND_TRACE_ID);
        result.mocking = true;
        result
    }

    pub fn new(trace_id: Id) -> Self {
        Self {
            trace_id,
            mocking: false,
            mocking_transaction: Arc::new(Mutex::new(false)),
            #[cfg(feature = "cloud")]
            transaction: Arc::new(Mutex::new(None)),
            version_for_transaction: Arc::new(Mutex::new(None)),
        }
    }

    // TODO 确保只在测试中使用 且名字不那么突兀
    pub fn mock_bind_transaction(&self, version_for_transaction: &Version) -> Result<()> {
        if self.mocking == false {
            return Unexpected!("mock_bind_transaction only supported in mocking");
        }
        if self.trace_id != Self::BACKGROUND_TRACE_ID {
            return Unexpected!("mock_bind_transaction only supported in background context");
        }
        let mut mocking_transaction_guard = self.mocking_transaction.try_lock()?;
        if *mocking_transaction_guard {
            return Unexpected!("transaction activated before mock");
        }
        *mocking_transaction_guard = true;

        let mut version_for_transaction_guard = self.version_for_transaction.try_lock()?;
        if version_for_transaction_guard.is_some() {
            return Unexpected!("version activated before bind");
        }
        *version_for_transaction_guard = Some(version_for_transaction.clone());
        Ok(())
    }

    #[cfg(feature = "cloud")]
    pub async fn bind_transaction(
        &self,
        transaction: Transaction<'static, Postgres>,
        version_for_transaction: &Version,
    ) -> Result<()> {
        if self.mocking {
            return Unexpected!("mocking context");
        }
        let mut transaction_guard = self.transaction.lock().await;
        if transaction_guard.is_some() {
            return Unexpected!("transaction activated before bind");
        }
        *transaction_guard = Some(transaction);

        let mut version_for_transaction_guard = self.version_for_transaction.lock().await;
        if version_for_transaction_guard.is_some() {
            return Unexpected!("version activated before bind");
        }
        *version_for_transaction_guard = Some(version_for_transaction.clone());
        Ok(())
    }

    // TODO 确保只在测试中使用 且名字不那么突兀
    pub fn in_mocking_transaction(&self) -> Result<bool> {
        if !self.mocking {
            return Unexpected!("non-mocking context");
        }
        Ok(*self.mocking_transaction.try_lock()?)
    }

    pub async fn in_transaction(&self) -> Result<bool> {
        if self.mocking {
            return Unexpected!("mocking transaction not supported");
        }

        #[cfg(feature = "cloud")]
        if self.transaction.lock().await.is_some() {
            return Ok(true);
        }

        Ok(false)
    }

    fn _rollback_version_for_transaction(&self) -> Result<()> {
        let mut version_for_transaction_guard = self.version_for_transaction.try_lock()?;
        if let Some(version_for_transaction) = version_for_transaction_guard.take() {
            version_for_transaction._decrease()?;
            *version_for_transaction_guard = None;
            Ok(())
        } else {
            Unexpected!("rollback non-version context")
        }
    }

    // 仅由 transaction 宏调用，保证框架层的内存数据回滚
    // 避免业务重试时使用未销毁的错误事务或者错误数据版本
    // 由于是 drop 时调用，因此无法使用 async
    fn _rollback_transaction(&self) -> Result<()> {
        if self.mocking {
            let mut mocking_transaction_guard = self.mocking_transaction.try_lock()?;
            if *mocking_transaction_guard == false {
                return Unexpected!("rollback non-transaction context");
            }
            *mocking_transaction_guard = false;
            self._rollback_version_for_transaction()?;
            return Ok(());
        }

        #[cfg(feature = "cloud")]
        {
            let mut transaction_guard = self.transaction.try_lock()?;
            if let Some(transaction) = transaction_guard.take() {
                // drop 会内部调用 begin_rollback, 避免直接 rollback 的await？
                // transaction.rollback().await?;
                drop(transaction);

                // 预防开启事务之后继续clone context带来的事务错误处理的可能。
                *transaction_guard = None;
            } else {
                return Unexpected!("rollback non-transaction context");
            }
        }

        self._rollback_version_for_transaction()
    }

    fn _commit_version_for_transaction(&self) -> Result<()> {
        // 由于外侧有 transaction 锁，因此version不应该出现冲突，不await
        let mut version_for_transaction_guard = self.version_for_transaction.try_lock()?;
        if let Some(_) = version_for_transaction_guard.take() {
            *version_for_transaction_guard = None;
            Ok(())
        } else {
            Unexpected!("commit non-version context")
        }
    }
    // 仅由 transaction 宏调用
    async fn _commit_transaction(&self) -> Result<()> {
        if self.mocking {
            let mut mocking_transaction_guard = self.mocking_transaction.try_lock()?;
            if *mocking_transaction_guard == false {
                return Unexpected!("commit non-transaction context");
            }
            *mocking_transaction_guard = false;

            self._commit_version_for_transaction()?;
            return Ok(());
        }

        #[cfg(feature = "cloud")]
        {
            let mut transaction_guard = self.transaction.lock().await;
            if let Some(transaction) = transaction_guard.take() {
                transaction.commit().await?;
                // 预防开启事务之后继续clone context带来的事务错误处理的可能。
                *transaction_guard = None;
            } else {
                return Unexpected!("commit non-transaction context");
            }
        }
        self._commit_version_for_transaction()?;
        Ok(())
    }

    // 此处 clone 用于transaction 宏，不会重新新建 RwLock
    fn _clone_for_transaction(&self) -> Result<Self> {
        if self.mocking {
            match self.mocking_transaction.try_lock() {
                Ok(mocking_transaction) => {
                    if *mocking_transaction {
                        // 阻止 transaction 进行clone
                        return Unexpected!(
                            "DO NOT clone Context for transaction in mocking_transaction"
                        );
                    }
                }
                Err(e) => {
                    return Unexpected!(
                        "DO NOT clone Context for transaction in mocking_transaction: {}",
                        e
                    );
                }
            }
        }

        #[cfg(feature = "cloud")]
        match self.transaction.try_lock() {
            Ok(transaction) => {
                if transaction.is_some() {
                    // 阻止 transaction 进行clone
                    return Unexpected!("DO NOT clone Context for transaction in transaction");
                }
            }
            Err(e) => {
                return Unexpected!("DO NOT clone Context for transaction in transaction: {}", e);
            }
        }

        match self.version_for_transaction.try_lock() {
            Ok(version) => {
                if version.is_some() {
                    return Unexpected!("DO NOT clone Context with version_for_transaction");
                }
            }
            Err(e) => {
                return Unexpected!("DO NOT clone Context with version_for_transaction: {}", e);
            }
        }

        Ok(Self {
            trace_id: self.trace_id.clone(),
            mocking: self.mocking,
            mocking_transaction: self.mocking_transaction.clone(),
            #[cfg(feature = "cloud")]
            transaction: self.transaction.clone(),
            version_for_transaction: self.version_for_transaction.clone(),
        })
    }
}
// 仅用于 transaction! 宏
pub struct _Transaction {
    committed: bool,
    context: Context,
}

impl _Transaction {
    pub fn from_context(context: &Context) -> Result<Self> {
        Ok(Self {
            committed: false,
            context: context._clone_for_transaction()?,
        })
    }

    pub async fn _commit(&mut self) -> Result<()> {
        self.context._commit_transaction().await?;
        self.committed = true;
        Ok(())
    }

    fn _rollback(&mut self) -> Result<()> {
        self.context._rollback_transaction()
    }
}

impl Drop for _Transaction {
    fn drop(&mut self) {
        if !self.committed {
            self._rollback().unwrap();
        }
    }
}
// 预期事务内应只有数据库操作和可重复无状态的纯计算，故错误都可以自由回滚
// 不回滚的操作应该进行错误处理和闭包变量控制
#[macro_export]
macro_rules! transaction {
    ($context:ident, $($body:tt)*) => {
        {
            let mut transaction = _Transaction::from_context($context)?;
            let result = { $($body)* };
            transaction._commit().await?;
            result
        }
    };
}
