#![feature(linkage)]

pub mod context;
pub mod instruction;
pub mod instruction_list;
pub mod mcontext;
pub mod module;
pub mod operand;

#[cfg(feature = "mgr")]
pub mod manager;

#[cfg(feature = "syms")]
pub mod symbols;

use atomic::{Atomic, Ordering};
use dynamorio_sys::*;
use std::ffi::{CStr, CString};

pub use context::{AfterSyscallContext, BeforeSyscallContext, Context};
pub use dynamorio_sys::{
    dr_emit_flags_t,
    dr_spill_slot_t,
};
pub use instruction::Instruction;
pub use instruction_list::InstructionList;
pub use mcontext::MachineContext;
pub use module::ModuleData;
pub use operand::{Operand, SourceOperandIter, TargetOperandIter};

#[cfg(feature = "mgr")]
pub use manager::Manager;

#[cfg(feature = "syms")]
pub use dynamorio_sys::drsym_flags_t;

#[cfg(feature = "syms")]
pub use symbols::Symbols;

/// We need to define `_USES_DR_VERSION_` as DynamoRIO checks this symbol for version
/// compatibility.
#[no_mangle]
pub static _USES_DR_VERSION_: std::os::raw::c_int = dynamorio_sys::_USES_DR_VERSION_;

/// We need to define `_DR_CLIENT_AVX512_CODE_IN_USE` as DynamoRIO checks this symbol to determine
/// whether AVX-512 is being used or not.
#[no_mangle]
pub static _DR_CLIENT_AVX512_CODE_IN_USE: std::os::raw::c_char = dynamorio_sys::_DR_CLIENT_AVX512_CODE_IN_USE_;

static EXIT_HANDLER: Atomic<Option<fn() -> ()>> = Atomic::new(None);

extern "C" fn event_exit() {
    if let Some(handler) = EXIT_HANDLER.load(Ordering::Relaxed) {
        handler()
    }
}

/// Registers a callback function for the process exit event. DynamoRIO calls `func` when the
/// process exits.
pub fn register_exit_event(func: fn() -> ()) {
    EXIT_HANDLER.store(Some(func), Ordering::Relaxed);

    unsafe {
        dr_register_exit_event(Some(event_exit));
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ClientId(pub client_id_t);

#[linkage = "weak"]
#[no_mangle]
fn client_main(_id: ClientId, _args: &[&str]) {
}

#[no_mangle]
fn dr_client_main(
    id: client_id_t, 
    argc: i32,
    argv: *const *const std::os::raw::c_char,
) {
    let id = ClientId(id);
    let args = unsafe {
        std::slice::from_raw_parts(
            argv,
            argc as _,
        )
    };

    let args: Vec<String> = args
        .into_iter()
        .map(|arg| unsafe { CStr::from_ptr(*arg) }
             .to_str()
             .unwrap()
             .to_owned())
        .collect();
    let args: Vec<&str> = args
        .iter()
        .map(|arg| &**arg)
        .collect();

    client_main(id, &args);
}

/// Sets information presented to users in diagnostic messages. Only one name is supported,
/// regardless of how many clients are in use. If this routine is called a second time, the new
/// values supersede the original. The `report_url` is meant to be a bug tracker location where
/// users should go to report errors in the client end-user tool.
pub fn set_client_name(name: &str, report_url: &str) {
    let name = CString::new(name).unwrap();
    let report_url = CString::new(report_url).unwrap();

    unsafe {
        dr_set_client_name(name.as_ptr(), report_url.as_ptr());
    }
}
