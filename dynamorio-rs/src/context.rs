use crate::{Instruction, MachineContext, Operand};
use dynamorio_sys::*;

pub struct Context {
    pub(crate) context: *mut core::ffi::c_void,
}

impl Context {
    pub fn from_raw(context: *mut core::ffi::c_void) -> Self {
        Self {
            context,
        }
    }

    pub fn raw(&self) -> *mut core::ffi::c_void {
        self.context
    }

    pub fn current() -> Self {
        let context = unsafe {
            dr_get_current_drcontext()
        };

        Self {
            context,
        }
    }

    pub fn thread_id(&self) -> thread_id_t {
        unsafe {
            dr_get_thread_id(self.context)
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
            _mcontext: mcontext,
        }
    }

    pub fn create_instruction(
        &self,
        opcode: u32,
        targets: &[Operand],
        sources: &[Operand],
    ) -> Option<Instruction> {
        let instruction = match (targets.len(), sources.len()) {
            (1, 1) => unsafe {
                instr_create_1dst_1src(
                    self.context,
                    opcode as i32,
                    targets[0].raw,
                    sources[0].raw,
                )
            }
            (1, 2) => unsafe {
                instr_create_1dst_2src(
                    self.context,
                    opcode as i32,
                    targets[0].raw,
                    sources[0].raw,
                    sources[1].raw,
                )
            }
            _ => return None,
        };

        if instruction.is_null() {
            return None;
        }

        Some(Instruction {
            context: self.context,
            raw: instruction,
        })
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

    /// # Safety
    /// It is up to the caller to ensure that reading this parameter is safe: this routine does not
    /// know the number of parameters for each system call, nor does it check whether this might
    /// read off the base of the stack.
    pub unsafe fn param(&self, index: usize) -> reg_t {
        dr_syscall_get_param(
            self.context.context,
            index as i32,
        )
    }

    /// # Safety
    /// It is up to the caller to ensure that writing this parameter is safe: this routine does not
    /// know the number of parameters for each system call, nor does it check whether this might
    /// write beyond the base of the stack.
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
