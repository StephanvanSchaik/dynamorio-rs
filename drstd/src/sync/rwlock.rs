use core::cell::UnsafeCell;
use core::ops::{Deref, DerefMut};
use dynamorio_sys::*;

use crate::error::Error;

pub struct RwLockReadGuard<'a, T: ?Sized + 'a> {
    lock: &'a RwLock<T>,
}

unsafe impl<T: ?Sized + Sync> Sync for RwLockReadGuard<'_, T> {}

impl<T: ?Sized> Deref for RwLockReadGuard<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.lock.data.get() }
    }
}

impl<T: ?Sized> Drop for RwLockReadGuard<'_, T> {
    fn drop(&mut self) {
        unsafe {
            dr_rwlock_read_unlock(self.lock.inner);
        }
    }
}

pub struct RwLockWriteGuard<'a, T: ?Sized + 'a> {
    lock: &'a RwLock<T>,
}

unsafe impl<T: ?Sized + Sync> Sync for RwLockWriteGuard<'_, T> {}

impl<T: ?Sized> Deref for RwLockWriteGuard<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.lock.data.get() }
    }
}

impl<T: ?Sized> DerefMut for RwLockWriteGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.lock.data.get() }
    }
}

impl<T: ?Sized> Drop for RwLockWriteGuard<'_, T> {
    fn drop(&mut self) {
        unsafe {
            dr_rwlock_write_unlock(self.lock.inner);
        }
    }
}

#[derive(Debug)]
pub struct RwLock<T: ?Sized> {
    inner: *mut core::ffi::c_void,
    data: UnsafeCell<T>,
}

unsafe impl<T: ?Sized + Send> Send for RwLock<T> {}
unsafe impl<T: ?Sized + Send + Sync> Sync for RwLock<T> {}

impl<T> RwLock<T> {
    pub fn new(data: T) -> Result<Self, acid_io::Error> {
        let inner = unsafe {
            dr_rwlock_create()
        };

        Ok(Self {
            inner,
            data: UnsafeCell::new(data),
        })
    }

    pub fn try_read(&mut self) -> Result<RwLockReadGuard<'_, T>, Error> {
        Err(Error::WouldBlock)
    }

    pub fn read(&self) -> Result<RwLockReadGuard<'_, T>, acid_io::Error> {
        unsafe {
            dr_rwlock_read_lock(self.inner);
        }

        Ok(RwLockReadGuard {
            lock: self,
        })
    }

    pub fn try_write(&mut self) -> Result<RwLockWriteGuard<'_, T>, Error> {
        let result = unsafe {
            dr_rwlock_write_trylock(self.inner) != 0
        };

        if !result {
            return Err(Error::WouldBlock);
        }

        Ok(RwLockWriteGuard {
            lock: self,
        })
    }

    pub fn write(&mut self) -> Result<RwLockWriteGuard<'_, T>, acid_io::Error> {
        unsafe {
            dr_rwlock_write_lock(self.inner);
        }

        Ok(RwLockWriteGuard {
            lock: self,
        })
    }
}

impl<T: ?Sized> Drop for RwLock<T> {
    fn drop(&mut self) {
        unsafe {
            dr_rwlock_destroy(self.inner);
        }
    }
}
