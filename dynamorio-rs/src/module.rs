use dynamorio_sys::*;

use std::ffi::CStr;

#[derive(Debug)]
pub struct ModuleData(*const module_data_t);

impl ModuleData {
    pub fn from_raw(raw: *const module_data_t) -> Self {
        Self(raw)
    }

    pub fn preferred_name(&self) -> Option<&str> {
        let s = unsafe {
            dr_module_preferred_name(self.0)
        };

        if s.is_null() {
            return None;
        }

        unsafe {
            CStr::from_ptr(s)
                .to_str()
                .ok()
        }
    }
}
