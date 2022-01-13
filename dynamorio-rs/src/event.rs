use atomic::{Atomic, Ordering};
use crate::Context;
use dynamorio_sys::*;

static EXIT_HANDLER: Atomic<Option<fn() -> ()>> = Atomic::new(None);
static FORK_HANDLER: Atomic<Option<fn(&mut Context) -> ()>> = Atomic::new(None);

extern "C" fn exit_event() {
    if let Some(handler) = EXIT_HANDLER.load(Ordering::Relaxed) {
        handler()
    }
}

extern "C" fn fork_event(context: *mut std::ffi::c_void) {
    let mut context = Context::from_raw(context);

    if let Some(handler) = FORK_HANDLER.load(Ordering::Relaxed) {
        handler(&mut context)
    }
}

/// Registers a callback function for the process exit event. DynamoRIO calls `func` when the
/// process exits.
pub fn register_exit_event(func: fn() -> ()) {
    EXIT_HANDLER.store(Some(func), Ordering::Relaxed);

    unsafe {
        dr_register_exit_event(Some(exit_event));
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
