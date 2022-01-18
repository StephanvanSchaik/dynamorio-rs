//! Useful synchronization primitives.

mod mutex;
mod rwlock;

pub use alloc::sync::{Arc, Weak};
pub use mutex::Mutex;
pub use rwlock::RwLock;
pub use spin::{Barrier, Once};
