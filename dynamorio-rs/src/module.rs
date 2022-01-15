use dynamorio_sys::*;

use cstr_core::{CStr, CString};

#[derive(Debug)]
pub struct ModuleData {
    pub(crate) raw: *mut module_data_t,
}

impl ModuleData {
    pub fn from_raw(raw: *mut module_data_t) -> Self {
        Self {
            raw,
        }
    }

    pub fn main_module() -> Self {
        let raw = unsafe {
            dr_get_main_module()
        };

        Self {
            raw,
        }
    }

    pub fn from_address(address: usize) -> Option<Self> {
        let raw = unsafe {
            dr_lookup_module(address as _)
        };

        if raw.is_null() {
            return None;
        }

        Some(Self {
            raw,
        })
    }

    pub fn start(&self) -> usize {
        unsafe {
            (*self.raw).__bindgen_anon_1.start as usize
        }
    }

    pub fn full_path(&self) -> Option<&str> {
        let s = unsafe { (*self.raw).full_path };

        unsafe {
            CStr::from_ptr(s).to_str().ok()
        }
    }

    pub fn preferred_name(&self) -> Option<&str> {
        let s = unsafe {
            dr_module_preferred_name(self.raw)
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

    pub fn get_proc_address(&self, name: &str) -> Option<unsafe extern "C" fn ()> {
        let name = CString::new(name).unwrap();

        unsafe {
            dr_get_proc_address((*self.raw).__bindgen_anon_1.handle, name.as_ptr())
        }
    }
}

impl Drop for ModuleData {
    fn drop(&mut self) {
        unsafe {
            dr_free_module_data(self.raw);
        }
    }
}

impl Clone for ModuleData {
    fn clone(&self) -> Self {
        let raw = unsafe {
            dr_copy_module_data(self.raw)
        };

        Self {
            raw,
        }
    }
}
