use crate::{Context, Manager};
use drstd::sync::{Arc, Mutex};
use dynamorio_sys::*;

pub trait ThreadHandler {
    fn create_thread(&mut self, context: &mut Context);
    fn exit_thread(&mut self, context: &mut Context);
}

pub struct RegisteredThreadHandler<T: ThreadHandler> {
    _handler: Arc<Mutex<T>>,
}

impl<T: ThreadHandler> Drop for RegisteredThreadHandler<T> {
    fn drop(&mut self) {
        unsafe {
            drmgr_unregister_thread_init_event_user_data(
                Some(thread_init_event::<T>),
            );
        }

        unsafe {
            drmgr_unregister_thread_exit_event_user_data(
                Some(thread_exit_event::<T>),
            );
        }
    }
}

extern "C" fn thread_init_event<T: ThreadHandler>(
    context: *mut core::ffi::c_void,
    user_data: *mut core::ffi::c_void,
) {
    let mut context = Context::from_raw(context);
    let handler = unsafe { &*(user_data as *mut Mutex<T>) };

    if let Ok(mut handler) = handler.lock() {
        handler.create_thread(&mut context);
    }
}

extern "C" fn thread_exit_event<T: ThreadHandler>(
    context: *mut core::ffi::c_void,
    user_data: *mut core::ffi::c_void,
) {
    let mut context = Context::from_raw(context);
    let handler = unsafe { &*(user_data as *mut Mutex<T>) };

    if let Ok(mut handler) = handler.lock() {
        handler.exit_thread(&mut context);
    }
}

impl Manager {
    pub fn register_thread_handler<T: ThreadHandler>(
        &self,
        handler: &Arc<Mutex<T>>,
    ) -> RegisteredThreadHandler<T> {
        unsafe {
            drmgr_register_thread_init_event_user_data(
                Some(thread_init_event::<T>),
                core::ptr::null_mut(),
                Arc::as_ptr(&handler) as *mut core::ffi::c_void,
            );
            drmgr_register_thread_exit_event_user_data(
                Some(thread_exit_event::<T>),
                core::ptr::null_mut(),
                Arc::as_ptr(&handler) as *mut core::ffi::c_void,
            );
        }

        RegisteredThreadHandler {
            _handler: Arc::clone(&handler),
        }
    }
}
