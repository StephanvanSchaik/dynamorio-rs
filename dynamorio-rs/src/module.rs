use dynamorio_sys::*;

use core::ops::Range;
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

    /// Looks up the module for the given address.
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

    pub fn end(&self) -> usize {
        unsafe {
            (*self.raw).end as usize
        }
    }

    pub fn range(&self) -> Range<usize> {
        self.start()..self.end()
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

    pub fn contains(&self, address: usize) -> bool {
        unsafe {
            dr_module_contains_addr(self.raw, address as app_pc) != 0
        }
    }

    pub fn get_proc_address(&self, name: &str) -> Option<unsafe extern "C" fn ()> {
        let name = CString::new(name).unwrap();

        unsafe {
            dr_get_proc_address((*self.raw).__bindgen_anon_1.handle, name.as_ptr())
        }
    }

    /// Returns whether the code from the module should be instrumented, i.e. whether it should be
    /// passed to the basic block event.
    pub fn instrumented(&self) -> bool {
        unsafe {
            dr_module_should_instrument((*self.raw).__bindgen_anon_1.handle) != 0
        }
    }

    /// Set whether or not the module should be instrumented. If `instrumented` is set to `false`,
    /// then the code from the module will not be passed to the basic block event.
    pub fn instrument(&mut self, instrumented: bool) {
        unsafe {
            dr_module_set_should_instrument((*self.raw).__bindgen_anon_1.handle, instrumented as i8);
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
