use crate::Instruction;
use dynamorio_sys::*;

#[derive(Clone, Copy, Debug)]
pub struct Operand {
    pub(crate) raw: opnd_t,
}

impl Operand {
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
