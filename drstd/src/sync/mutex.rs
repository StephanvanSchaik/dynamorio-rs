use core::cell::UnsafeCell;
use core::ops::{Deref, DerefMut};
use dynamorio_sys::*;

use crate::error::Error;

pub struct MutexGuard<'a, T: ?Sized + 'a> {
    lock: &'a Mutex<T>,
}

unsafe impl<T: ?Sized + Sync> Sync for MutexGuard<'_, T> {}

impl<T: ?Sized> Deref for MutexGuard<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.lock.data.get() }
    }
}

impl<T: ?Sized> DerefMut for MutexGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.lock.data.get() }
    }
}

impl<T: ?Sized> Drop for MutexGuard<'_, T> {
    fn drop(&mut self) {
        unsafe {
            dr_mutex_unlock(self.lock.inner);
        }
    }
}

#[derive(Debug)]
pub struct Mutex<T: ?Sized> {
    inner: *mut core::ffi::c_void,
    data: UnsafeCell<T>,
}

unsafe impl<T: ?Sized + Send> Send for Mutex<T> {}
unsafe impl<T: ?Sized + Send + Sync> Sync for Mutex<T> {}

impl<T> Mutex<T> {
    pub fn new(data: T) -> Self {
        let inner = unsafe {
            dr_mutex_create()
        };

        Self {
            inner,
            data: UnsafeCell::new(data),
        }
    }

    pub fn try_lock(&mut self) -> Result<MutexGuard<'_, T>, Error> {
        let result = unsafe {
            dr_mutex_trylock(self.inner) != 0
        };

        if !result {
            return Err(Error::WouldBlock);
        }

        Ok(MutexGuard {
            lock: self,
        })
    }

    pub fn lock(&mut self) -> Result<MutexGuard<'_, T>, acid_io::Error> {
        unsafe {
            dr_mutex_lock(self.inner);
        }

        Ok(MutexGuard {
            lock: self,
        })
    }
}

impl<T: ?Sized> Drop for Mutex<T> {
    fn drop(&mut self) {
        unsafe {
            dr_mutex_destroy(self.inner);
        }
    }
}
