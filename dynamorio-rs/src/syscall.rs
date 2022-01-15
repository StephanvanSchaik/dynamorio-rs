use atomic::{Atomic, Ordering};
use crate::Context;
use dynamorio_sys::*;

static FILTER_SYSCALL_HANDLER: Atomic<Option<fn(&mut Context, i32) -> bool>> = Atomic::new(None);

extern "C" fn filter_syscall_event(context: *mut core::ffi::c_void, sysnum: i32) -> i8 {
    let mut context = Context::from_raw(context);
    let mut result = false;

    if let Some(handler) = FILTER_SYSCALL_HANDLER.load(Ordering::Relaxed) {
        result = handler(&mut context, sysnum);
    }

    result as i8
}

pub fn register_filter_syscall_event(
    func: fn(&mut Context, i32) -> bool,
) {
    FILTER_SYSCALL_HANDLER.store(Some(func), Ordering::Relaxed);

    unsafe {
        dr_register_filter_syscall_event(Some(filter_syscall_event));
    }
}
