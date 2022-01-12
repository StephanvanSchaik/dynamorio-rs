use dynamorio_sys::*;

use atomic::{Atomic, Ordering};
use crate::module::ModuleData;

static MODULE_LOAD_HANDLER: Atomic<Option<fn(ModuleData, bool) -> ()>> = Atomic::new(None);

extern "C" fn module_load_event(
    drcontext: *mut std::ffi::c_void,
    module: *const module_data_t,
    loaded: std::os::raw::c_char,
) {
    if let Some(handler) = MODULE_LOAD_HANDLER.load(Ordering::Relaxed) {
        handler(
            ModuleData::from_raw(module),
            loaded != 0,
        );
    }
}

pub struct Manager;

impl Manager {
    pub fn new() -> Self {
        unsafe {
            drmgr_init();
        }

        Self
    }

    pub fn register_module_load_event(&self, func: fn(ModuleData, bool) -> ()) {
        MODULE_LOAD_HANDLER.store(Some(func), Ordering::Relaxed);

        unsafe {
            drmgr_register_module_load_event(Some(module_load_event));
        }
    }
}
