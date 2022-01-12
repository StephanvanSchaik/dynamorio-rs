use crate::{Instruction, InstructionList, MachineContext};
use dynamorio_sys::*;

pub struct Context {
    context: *mut std::ffi::c_void,
}

impl Context {
    pub fn from_raw(context: *mut std::ffi::c_void) -> Self {
        Self {
            context,
        }
    }

    pub fn current() -> Self {
        let context = unsafe {
            dr_get_current_drcontext()
        };

        Self {
            context,
        }
    }

    pub fn read_saved_register(&self, register: dr_spill_slot_t) -> reg_t {
        unsafe {
            dr_read_saved_reg(self.context, register)
        }
    }

    pub fn write_saved_register(&mut self, register: dr_spill_slot_t, value: reg_t) {
        unsafe {
            dr_write_saved_reg(self.context, register, value)
        }
    }

    pub fn using_app_state(&mut self) -> bool {
        unsafe {
            dr_using_app_state(self.context) != 0
        }
    }

    pub fn switch_to_app_state(&mut self) {
        unsafe {
            dr_switch_to_app_state(self.context);
        }
    }

    pub fn switch_to_dr_state(&mut self) {
        unsafe {
            dr_switch_to_dr_state(self.context);
        }
    }

    pub fn get_machine_context(&self, flags: dr_mcontext_flags_t) -> MachineContext {
        let mut mcontext: dr_mcontext_t = unsafe {
            std::mem::zeroed()
        };

        mcontext.size = std::mem::size_of::<dr_mcontext_t>();
        mcontext.flags = flags;

        unsafe {
            dr_get_mcontext(self.context, &mut mcontext);
        }

        MachineContext {
            mcontext,
        }
    }
}
