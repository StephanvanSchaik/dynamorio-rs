use atomic::{Atomic, Ordering};
use crate::{ClientId, Context};
use dynamorio_sys::*;

static EXIT_HANDLER: Atomic<Option<fn() -> ()>> = Atomic::new(None);
static FORK_HANDLER: Atomic<Option<fn(&mut Context) -> ()>> = Atomic::new(None);
static NUDGE_HANDLER: Atomic<Option<fn(&mut Context, u64) -> ()>> = Atomic::new(None);

extern "C" fn exit_event() {
    if let Some(handler) = EXIT_HANDLER.load(Ordering::Relaxed) {
        handler()
    }
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

pub fn register_nudge_event(
    func: fn(&mut Context, u64) -> (),
    client_id: ClientId,
) {
    NUDGE_HANDLER.store(Some(func), Ordering::Relaxed);

    unsafe {
        dr_register_nudge_event(Some(nudge_event), client_id.0);
    }
}
