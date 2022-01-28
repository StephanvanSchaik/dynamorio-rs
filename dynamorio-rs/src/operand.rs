use crate::Instruction;
use dynamorio_sys::*;

#[derive(Clone, Copy, Debug)]
pub struct Operand {
    pub(crate) raw: opnd_t,
}

impl Operand {
    pub fn new_register(register: reg_id_t) -> Self {
        let raw = unsafe {
            opnd_create_reg(register)
        };

        Self {
            raw,
        }
    }

    pub fn new_immediate(value: u64, operand_size: opnd_size_t) -> Self {
        let raw = unsafe {
            opnd_create_immed_uint(value, operand_size)
        };

        Self {
            raw,
        }
    }

    pub fn new_memptr(base: reg_id_t, displacement: i32) -> Self {
        let raw = unsafe {
            opnd_create_base_disp(base, DR_REG_NULL as _, 0, displacement, OPSZ_8 as _)
        };

        Self {
            raw,
        }
    }

    pub fn is_memory_reference(&self) -> bool {
        unsafe {
            opnd_is_memory_reference(self.raw) != 0
        }
    }

    pub fn displacement(&self) -> i32 {
        unsafe {
            opnd_get_disp(self.raw)
        }
    }

    pub fn register(&self) -> Option<reg_id_t> {
        let register = unsafe {
            opnd_get_reg(self.raw)
        };

        if register == DR_REG_NULL as u16 {
            return None;
        }

        Some(register)
    }

    pub fn segment(&self) -> Option<reg_id_t> {
        let register = unsafe {
            opnd_get_segment(self.raw)
        };

        if register == DR_REG_NULL as u16 {
            return None;
        }

        Some(register)
    }
}

#[derive(Debug)]
pub struct SourceOperandIter<'a> {
    pub(crate) instruction: &'a Instruction,
    pub(crate) index: usize,
    pub(crate) count: usize,
}

impl<'a> Iterator for SourceOperandIter<'a> {
    type Item = Operand;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.count {
            return None;
        }

        let raw = unsafe {
            instr_get_src(self.instruction.raw, self.index as _)
        };

        self.index += 1;

        Some(Operand {
            raw,
        })
    }
}

#[derive(Debug)]
pub struct TargetOperandIter<'a> {
    pub(crate) instruction: &'a Instruction,
    pub(crate) index: usize,
    pub(crate) count: usize,
}

impl<'a> Iterator for TargetOperandIter<'a> {
    type Item = Operand;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.count {
            return None;
        }

        let raw = unsafe {
            instr_get_src(self.instruction.raw, self.index as _)
        };

        self.index += 1;

        Some(Operand {
            raw,
        })
    }
}
