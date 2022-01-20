use crate::MachineContext;
use dynamorio_sys::*;

pub struct Context {
    context: *mut core::ffi::c_void,
}

impl Context {
    pub fn from_raw(context: *mut core::ffi::c_void) -> Self {
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
            core::mem::zeroed()
        };

        mcontext.size = core::mem::size_of::<dr_mcontext_t>();
        mcontext.flags = flags;

        unsafe {
            dr_get_mcontext(self.context, &mut mcontext);
        }

        MachineContext {
            mcontext,
        }
    }
}

pub struct BeforeSyscallContext {
    context: Context,
}

impl BeforeSyscallContext {
    pub fn from_raw(context: *mut core::ffi::c_void) -> Self {
        Self {
            context: Context::from_raw(context),
        }
    }

    pub fn context(&self) -> &Context {
        &self.context
    }

    pub fn context_mut(&mut self) -> &mut Context {
        &mut self.context
    }

    pub unsafe fn param(&self, index: usize) -> reg_t {
        dr_syscall_get_param(
            self.context.context,
            index as i32,
        )
    }

    pub unsafe fn set_param(&mut self, index: usize, value: reg_t) {
        dr_syscall_set_param(
            self.context.context,
            index as i32,
            value,
        );
    }

    pub fn set_sysnum(&mut self, sysnum: i32) {
        unsafe {
            dr_syscall_set_sysnum(
                self.context.context,
                sysnum,
            );
        }
    }
}

pub struct AfterSyscallContext {
    context: Context,
}

impl AfterSyscallContext {
    pub fn from_raw(context: *mut core::ffi::c_void) -> Self {
        Self {
            context: Context::from_raw(context),
        }
    }

    pub fn context(&self) -> &Context {
        &self.context
    }

    pub fn context_mut(&mut self) -> &mut Context {
        &mut self.context
    }

    pub fn get_result(&self) -> reg_t {
        unsafe {
            dr_syscall_get_result(
                self.context.context,
            )
        }
    }
}
