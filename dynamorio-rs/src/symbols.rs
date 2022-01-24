use alloc::borrow::ToOwned;
use alloc::string::String;
use alloc::vec;
use cstr_core::CStr;
use dynamorio_sys::*;

pub struct Symbols;

impl Symbols {
    pub fn new() -> Self {
        unsafe {
            drsym_init(0);
        }

        Self
    }

    pub fn lookup_address(
        &self,
        path: &str,
        offset: usize,
        flags: drsym_flags_t,
    ) -> Option<String> {
        let mut name = vec![0i8; 256];
        let mut file_name = vec![0i8; 256];
        let mut sym_info: drsym_info_t = unsafe {
            core::mem::zeroed()
        };

        sym_info.struct_size = core::mem::size_of::<drsym_info_t>();
        sym_info.name = name.as_mut_ptr();
        sym_info.name_size = name.len() as _;
        sym_info.file = file_name.as_mut_ptr();
        sym_info.file_size = file_name.len() as _;

        let result = unsafe {
            drsym_lookup_address(
                path.as_ptr() as _,
                offset,
                &mut sym_info,
                flags.0,
            )
        };

        if result != drsym_error_t::DRSYM_SUCCESS {
            return None;
        }

        unsafe {
            CStr::from_ptr(sym_info.name)
                 .to_str()
                 .map(|s| s.to_owned())
                 .ok()
        }
    }
}
