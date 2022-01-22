use atomic::{Atomic, Ordering};
use crate::{ClientId, Context};
use drstd::sync::{Arc, Mutex};
use dynamorio_sys::*;

static FORK_HANDLER: Atomic<Option<fn(&mut Context) -> ()>> = Atomic::new(None);
static NUDGE_HANDLER: Atomic<Option<fn(&mut Context, u64) -> ()>> = Atomic::new(None);

pub trait ExitHandler {
    fn exit(&mut self);
}

extern "C" fn fork_event(context: *mut core::ffi::c_void) {
    let mut context = Context::from_raw(context);

    if let Some(handler) = FORK_HANDLER.load(Ordering::Relaxed) {
        handler(&mut context)
    }
}

extern "C" fn nudge_event(context: *mut core::ffi::c_void, argument: u64) {
    let mut context = Context::from_raw(context);

    if let Some(handler) = NUDGE_HANDLER.load(Ordering::Relaxed) {
        handler(&mut context, argument)
    }
}

unsafe extern "C" fn exit_wrapper<T: ExitHandler>(
    handler: &Mutex<T>,
) {
    if let Ok(mut handler) = handler.lock() {
        handler.exit()
    }
}

pub struct RegisteredExitHandler<T: ExitHandler> {
    _handler: Arc<Mutex<T>>,
    func: extern "C" fn(),
    closure: *mut core::ffi::c_void,
}

unsafe impl<T: ExitHandler> Send for RegisteredExitHandler<T> {}
unsafe impl<T: ExitHandler> Sync for RegisteredExitHandler<T> {}

impl<T: ExitHandler> Drop for RegisteredExitHandler<T> {
    fn drop(&mut self) {
        unsafe {
            dr_unregister_exit_event(Some(self.func));
        }

        unsafe {
            dr_nonheap_free(self.closure, 32);
        }
    }
}

/// Registers a callback function for the process exit event. DynamoRIO calls `func` when the
/// process exits.
pub fn register_exit_handler<T: ExitHandler>(handler: &Arc<Mutex<T>>) -> RegisteredExitHandler<T> {
    let closure = unsafe {
        dr_nonheap_alloc(32, DR_MEMPROT_READ | DR_MEMPROT_WRITE | DR_MEMPROT_EXEC)
    };

    let storage: &mut [u8] = unsafe {
        core::slice::from_raw_parts_mut(closure as *mut u8, 32)
    };
    let code = &[
        0x48, 0x8b, 0x3d, 0xe9, 0xff, 0xff, 0xff,
        0xff, 0x25, 0xeb, 0xff, 0xff, 0xff,
    ];

    storage[16..16 + 13].copy_from_slice(code);

    let storage: &mut [u64] = unsafe {
        core::slice::from_raw_parts_mut(closure as *mut u64, 2)
    };

    storage[0] = Arc::as_ptr(&handler) as u64;
    storage[1] = exit_wrapper::<T> as u64;

    let func: extern "C" fn() = unsafe {
        core::mem::transmute((closure as *mut u8).add(16))
    };

    unsafe {
        dr_register_exit_event(Some(func));
    }

    RegisteredExitHandler {
        _handler: Arc::clone(&handler),
        func,
        closure,
    }
}

#[cfg(unix)]
pub fn register_fork_event(func: fn(&mut Context) -> ()) {
    FORK_HANDLER.store(Some(func), Ordering::Relaxed);

    unsafe {
        dr_register_fork_init_event(Some(fork_event));
    }
}

#[cfg(not(unix))]
pub fn register_fork_event(func: fn(&mut Context) -> ()) {
}

pub fn register_nudge_event(
    func: fn(&mut Context, u64) -> (),
    client_id: ClientId,
) {
    NUDGE_HANDLER.store(Some(func), Ordering::Relaxed);

    unsafe {
        dr_register_nudge_event(Some(nudge_event), client_id.0);
    }
}
