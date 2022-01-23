use crate::Manager;
use crate::{AfterSyscallContext, BeforeSyscallContext, Context};
use crate::closure::Closure;
use drstd::sync::{Arc, Mutex};
use dynamorio_sys::*;

pub trait SyscallHandler {
    fn filter_syscall(&mut self, context: &mut Context, sysno: i32) -> bool;
    fn before_syscall(&mut self, context: &mut BeforeSyscallContext, sysno: i32) -> bool;
    fn after_syscall(&mut self, context: &mut AfterSyscallContext, sysno: i32);
}

pub struct RegisteredSyscallHandler<T: SyscallHandler> {
    _handler: Arc<Mutex<T>>,
    closure: Closure,
}

unsafe impl<T: SyscallHandler> Send for RegisteredSyscallHandler<T> {}
unsafe impl<T: SyscallHandler> Sync for RegisteredSyscallHandler<T> {}

impl<T: SyscallHandler> Drop for RegisteredSyscallHandler<T> {
    fn drop(&mut self) {
        let func: extern "C" fn(*mut core::ffi::c_void, i32) -> i8 = unsafe {
            core::mem::transmute(self.closure.code())
        };

        unsafe {
            dr_unregister_filter_syscall_event(
                Some(func),
            );
            drmgr_unregister_pre_syscall_event_user_data(
                Some(before_syscall_event::<T>),
            );
            drmgr_unregister_post_syscall_event_user_data(
                Some(after_syscall_event::<T>),
            );
        }
    }
}

extern "C" fn filter_syscall_event<T: SyscallHandler>(
    context: *mut core::ffi::c_void,
    sysnum: i32,
    handler: &Mutex<T>,
) -> i8 {
    let mut context = Context::from_raw(context);
    let mut result = false;

    if let Ok(mut handler) = handler.lock() {
        result = handler.filter_syscall(&mut context, sysnum);
    }

    result as i8
}

extern "C" fn before_syscall_event<T: SyscallHandler>(
    context: *mut core::ffi::c_void,
    sysnum: i32,
    user_data: *mut core::ffi::c_void,
) -> i8 {
    let mut context = BeforeSyscallContext::from_raw(context);
    let handler = unsafe { &*(user_data as *mut Mutex<T>) };
    let mut result = 0;

    if let Ok(mut handler) = handler.lock() {
        result = handler.before_syscall(&mut context, sysnum) as i8;
    }

    result
}

extern "C" fn after_syscall_event<T: SyscallHandler>(
    context: *mut core::ffi::c_void,
    sysnum: i32,
    user_data: *mut core::ffi::c_void,
) {
    let mut context = AfterSyscallContext::from_raw(context);
    let handler = unsafe { &*(user_data as *mut Mutex<T>) };

    if let Ok(mut handler) = handler.lock() {
        handler.after_syscall(&mut context, sysnum);
    }
}

impl Manager {
    #[cfg(target_os = "windows")]
    pub fn decode_sysnum_from_wrapper(
        &self,
        entry: usize,
    ) -> Option<u32> {
        let result = unsafe {
            drmgr_decode_sysnum_from_wrapper(entry)
        };

        if result < 0 {
            return None;
        }

        Some(result as u32)
    }

    pub fn register_syscall_handler<T: SyscallHandler>(
        &self,
        handler: &Arc<Mutex<T>>,
    ) -> RegisteredSyscallHandler<T> {
        let closure = Closure::new(
            2,
            unsafe {
                core::mem::transmute(filter_syscall_event::<T> as extern "C" fn(_, _, _) -> _)
            },
            Arc::as_ptr(&handler) as *mut core::ffi::c_void,
        );

        let func: extern "C" fn(*mut core::ffi::c_void, i32) -> i8 = unsafe {
            core::mem::transmute(closure.code())
        };

        unsafe {
            dr_register_filter_syscall_event(Some(func));
            drmgr_register_pre_syscall_event_user_data(
                Some(before_syscall_event::<T>),
                core::ptr::null_mut(),
                Arc::as_ptr(&handler) as *mut core::ffi::c_void,
            );
            drmgr_register_post_syscall_event_user_data(
                Some(after_syscall_event::<T>),
                core::ptr::null_mut(),
                Arc::as_ptr(&handler) as *mut core::ffi::c_void,
            );
        }

        RegisteredSyscallHandler {
            _handler: Arc::clone(&handler),
            closure,
        }
    }
}
