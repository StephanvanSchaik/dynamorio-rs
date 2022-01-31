use crate::{Operand, SourceOperandIter, TargetOperandIter};
use dynamorio_sys::*;

#[derive(Debug)]
pub struct Instruction {
    pub(crate) context: *mut core::ffi::c_void,
    pub(crate) raw: *mut instr_t,
}

impl Instruction {
    pub fn from_raw(context: *mut core::ffi::c_void, raw: *mut instr_t) -> Self {
        Self {
            context,
            raw,
        }
    }

    pub fn raw(&self) -> *mut instr_t {
        self.raw
    }

    pub fn location(&self) -> usize {
        unsafe {
            instr_get_app_pc(self.raw) as usize
        }
    }

    pub fn opcode(&self) -> u32 {
        unsafe {
            instr_get_opcode(self.raw) as u32
        }
    }

    pub fn is_direct_call(&self) -> bool {
        unsafe {
            instr_is_call_direct(self.raw) != 0
        }
    }

    pub fn is_indirect_call(&self) -> bool {
        unsafe {
            instr_is_call_indirect(self.raw) != 0
        }
    }

    pub fn is_return(&self) -> bool {
        unsafe {
            instr_is_return(self.raw) != 0
        }
    }

    pub fn reads_memory(&self) -> bool {
        unsafe {
            instr_reads_memory(self.raw) != 0
        }
    }

    pub fn writes_memory(&self) -> bool {
        unsafe {
            instr_writes_memory(self.raw) != 0
        }
    }

    pub fn replace_source_operands(&self, operand: Operand, new_operand: Operand) -> bool {
        unsafe {
            instr_replace_src_opnd(self.raw, operand.raw, new_operand.raw) != 0
        }
    }

    pub fn source_operands(&self) -> SourceOperandIter {
        let count = unsafe {
            instr_num_srcs(self.raw)
        } as usize;

        SourceOperandIter {
            instruction: self,
            index: 0,
            count,
        }
    }

    pub fn target_operands(&self) -> TargetOperandIter {
        let count = unsafe {
            instr_num_dsts(self.raw)
        } as usize;

        TargetOperandIter {
            instruction: self,
            index: 0,
            count,
        }
    }
}

impl Drop for Instruction {
    fn drop(&mut self) {
        unsafe {
            instr_destroy(self.context, self.raw);
        }
    }
}

impl Clone for Instruction {
    fn clone(&self) -> Self {
        let raw = unsafe {
            instr_clone(self.context, self.raw)
        };

        Self {
            context: self.context,
            raw,
        }
    }
}
