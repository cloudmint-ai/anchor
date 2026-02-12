use crate::*;
use std::any::type_name;
use std::ops::{Deref, DerefMut};
use tokio::sync::{
    RwLock as TokioRwLock, RwLockReadGuard as TokioRwLockReadGuard,
    RwLockWriteGuard as TokioRwLockWriteGuard,
};

pub struct RwLock<T>(TokioRwLock<T>);

impl<T> RwLock<T> {
    pub fn new(data: T) -> Self {
        Self(TokioRwLock::new(data))
    }
    pub async fn read(&self) -> RwLockReadGuard<'_, T> {
        debug!("read locking [{}]", type_name::<T>());
        let guard = self.0.read().await;
        debug!("read locked [{}]", type_name::<T>());
        RwLockReadGuard(guard)
    }

    pub async fn write(&self) -> RwLockWriteGuard<'_, T> {
        debug!("write locking [{}]", type_name::<T>());
        let guard = self.0.write().await;
        debug!("write locked [{}]", type_name::<T>());
        RwLockWriteGuard(guard)
    }

    pub fn into_inner(self) -> T {
        self.0.into_inner()
    }

    pub fn try_read(&self) -> Result<RwLockReadGuard<'_, T>> {
        debug!("try read locking [{}]", type_name::<T>());
        let guard = self.0.try_read()?;
        debug!("try read locked [{}]", type_name::<T>());
        Ok(RwLockReadGuard(guard))
    }

    pub fn try_write(&self) -> Result<RwLockWriteGuard<'_, T>> {
        debug!("try write locking [{}]", type_name::<T>());
        let guard = self.0.try_write()?;
        debug!("try write locked [{}]", type_name::<T>());
        Ok(RwLockWriteGuard(guard))
    }
}

pub struct RwLockReadGuard<'a, T>(TokioRwLockReadGuard<'a, T>);

impl<'a, T> Drop for RwLockReadGuard<'a, T> {
    fn drop(&mut self) {
        debug!("read unlocked [{}]", type_name::<T>());
    }
}

impl<'a, T> Deref for RwLockReadGuard<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub struct RwLockWriteGuard<'a, T>(TokioRwLockWriteGuard<'a, T>);

impl<'a, T> Drop for RwLockWriteGuard<'a, T> {
    fn drop(&mut self) {
        debug!("write unlocked [{}]", type_name::<T>());
    }
}

impl<'a, T> Deref for RwLockWriteGuard<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &*self.0
    }
}

impl<'a, T> DerefMut for RwLockWriteGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug)]
    struct TestStruct {
        i: i64,
    }

    impl TestStruct {
        fn increase(&mut self) {
            self.i = self.i + 1
        }
    }

    #[test::case]
    async fn test_rw_lock() {
        let rw_lock = Arc::new(RwLock::new(TestStruct { i: 0 }));
        assert_eq!(rw_lock.read().await.i, 0);
        rw_lock.write().await.increase();
        assert_eq!(rw_lock.read().await.i, 1);
    }
}
