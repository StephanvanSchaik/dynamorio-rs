use crate::{Context, Instruction, Operand};
use dynamorio_sys::*;

impl Context {
    pub fn create_mov_ld(&mut self, dst: Operand, src: Operand) -> Instruction {
        let raw = unsafe {
            instr_create_1dst_1src(self.context, OP_mov_ld as _, dst.raw, src.raw)
        };

        Instruction {
            context: self.context,
            raw,
        }
    }

    pub fn create_mov_imm(&mut self, dst: Operand, src: Operand) -> Instruction {
        let raw = unsafe {
            instr_create_1dst_1src(self.context, OP_mov_imm as _, dst.raw, src.raw)
        };

        Instruction {
            context: self.context,
            raw,
        }
    }
}
