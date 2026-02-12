use crate::*;
use std::any::type_name;
use std::ops::{Deref, DerefMut};
use tokio::sync::{Mutex as TokioMutex, MutexGuard as TokioMutexGuard};

pub struct Mutex<T>(TokioMutex<T>);

impl<T> Mutex<T> {
    pub fn new(data: T) -> Self {
        Mutex(TokioMutex::new(data))
    }

    pub async fn lock(&self) -> MutexGuard<'_, T> {
        debug!("locking [{}]", type_name::<T>());
        let guard = self.0.lock().await;
        debug!("locked [{}]", type_name::<T>());
        MutexGuard(guard)
    }

    pub fn try_lock(&self) -> Result<MutexGuard<'_, T>> {
        debug!("try locking [{}]", type_name::<T>());
        let guard = self.0.try_lock()?;
        debug!("try locked [{}]", type_name::<T>());
        Ok(MutexGuard(guard))
    }
}

pub struct MutexGuard<'a, T>(TokioMutexGuard<'a, T>);

impl<'a, T> Deref for MutexGuard<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &*self.0
    }
}

impl<'a, T> DerefMut for MutexGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut *self.0
    }
}

impl<'a, T> Drop for MutexGuard<'a, T> {
    fn drop(&mut self) {
        debug!("unlocked [{:?}]", type_name::<T>());
    }
}

impl<'a, T> AsRef<T> for MutexGuard<'a, T> {
    fn as_ref(&self) -> &T {
        &*self.0
    }
}
impl<'a, T> AsMut<T> for MutexGuard<'a, T>
where
    T: AsMut<T>,
{
    fn as_mut(&mut self) -> &mut T {
        &mut *self.0
    }
}
