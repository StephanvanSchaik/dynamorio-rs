use atomic::{Atomic, Ordering};
use crate::{ClientId, Context};
use drstd::sync::{Arc, Mutex};
use dynamorio_sys::*;
use libffi::low::*;

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
    _cif: &ffi_cif,
    _result: &mut u64,
    _args: *const *const core::ffi::c_void,
    handler: &Mutex<T>,
) {
    if let Ok(mut handler) = handler.lock() {
        handler.exit()
    }
}

pub struct RegisteredExitHandler<T: ExitHandler> {
    _handler: Arc<Mutex<T>>,
    func: extern "C" fn(),
    closure: *mut ffi_closure,
}

unsafe impl<T: ExitHandler> Send for RegisteredExitHandler<T> {}
unsafe impl<T: ExitHandler> Sync for RegisteredExitHandler<T> {}

impl<T: ExitHandler> Drop for RegisteredExitHandler<T> {
    fn drop(&mut self) {
        unsafe {
            dr_unregister_exit_event(Some(self.func));
        }

        unsafe {
            closure_free(self.closure);
        }
    }
}

/// Registers a callback function for the process exit event. DynamoRIO calls `func` when the
/// process exits.
pub fn register_exit_handler<T: ExitHandler>(handler: &Arc<Mutex<T>>) -> RegisteredExitHandler<T> {
    let mut cif: ffi_cif = Default::default();

    unsafe {
        prep_cif(&mut cif, ffi_abi_FFI_DEFAULT_ABI, 0, &mut types::uint64, core::ptr::null_mut()).unwrap();
    }

    let (closure, code) = closure_alloc();
    let func: extern "C" fn() = unsafe {
        core::mem::transmute(code)
    };

    unsafe {
        prep_closure(
            closure,
            &mut cif,
            exit_wrapper::<T>,
            Arc::as_ptr(&handler),
            CodePtr(func as *mut _),
        ).unwrap();
    }

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
