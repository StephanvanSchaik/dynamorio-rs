use crate::{Context, Manager, ModuleData};
use drstd::sync::{Arc, Mutex};
use dynamorio_sys::*;

pub trait ModuleHandler {
    fn load_module(&mut self, context: &mut Context, module: &ModuleData, loaded: bool);
    fn unload_module(&mut self, context: &mut Context, module: &ModuleData);
}

pub struct RegisteredModuleHandler<T: ModuleHandler> {
    _handler: Arc<Mutex<T>>,
}

impl<T: ModuleHandler> Drop for RegisteredModuleHandler<T> {
    fn drop(&mut self) {
        unsafe {
            drmgr_unregister_module_load_event_user_data(
                Some(module_load_event::<T>),
            );
        }

        unsafe {
            drmgr_unregister_module_unload_event_user_data(
                Some(module_unload_event::<T>),
            );
        }
    }
}

extern "C" fn module_load_event<T: ModuleHandler>(
    context: *mut core::ffi::c_void,
    module: *const module_data_t,
    loaded: i8,
    user_data: *mut core::ffi::c_void,
) {
    let module = ModuleData::from_raw(module as _);
    let loaded = loaded != 0;
    let mut context = Context::from_raw(context);
    let handler = unsafe { &*(user_data as *mut Mutex<T>) };

    if let Ok(mut handler) = handler.lock() {
        handler.load_module(&mut context, &module, loaded);
    }

    core::mem::forget(context);
    core::mem::forget(module);
}

extern "C" fn module_unload_event<T: ModuleHandler>(
    context: *mut core::ffi::c_void,
    module: *const module_data_t,
    user_data: *mut core::ffi::c_void,
) {
    let module = ModuleData::from_raw(module as _);
    let mut context = Context::from_raw(context);
    let handler = unsafe { &*(user_data as *mut Mutex<T>) };

    if let Ok(mut handler) = handler.lock() {
        handler.unload_module(&mut context, &module);
    }

    core::mem::forget(context);
    core::mem::forget(module);
}

impl Manager {
    pub fn register_module_handler<T: ModuleHandler>(
        &self,
        handler: &Arc<Mutex<T>>,
    ) -> RegisteredModuleHandler<T> {
        unsafe {
            drmgr_register_module_load_event_user_data(
                Some(module_load_event::<T>),
                core::ptr::null_mut(),
                Arc::as_ptr(&handler) as *mut core::ffi::c_void,
            );
        }

        unsafe {
            drmgr_register_module_unload_event_user_data(
                Some(module_unload_event::<T>),
                core::ptr::null_mut(),
                Arc::as_ptr(&handler) as *mut core::ffi::c_void,
            );
        }

        RegisteredModuleHandler {
            _handler: Arc::clone(&handler),
        }
    }
}
