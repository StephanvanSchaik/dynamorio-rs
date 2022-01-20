use crate::instruction::Instruction;
use dynamorio_sys::*;

#[derive(Debug)]
pub struct InstructionList {
    pub(crate) context: *mut core::ffi::c_void,
    pub(crate) raw: *mut instrlist_t,
}

impl InstructionList {
    pub fn from_raw(context: *mut core::ffi::c_void, raw: *mut instrlist_t) -> Self {
        Self {
            context,
            raw,
        }
    }

    pub fn insert_before(
        &mut self,
        anchor: &Instruction,
        instruction: Instruction,
    ) {
        unsafe {
            instrlist_preinsert(self.raw, anchor.raw, instruction.raw);
        }
    }

    pub fn insert_after(
        &mut self,
        anchor: &Instruction,
        instruction: Instruction,
    ) {
        unsafe {
            instrlist_postinsert(self.raw, anchor.raw, instruction.raw);
        }
    }

    pub fn replace(
        &mut self,
        old_instruction: &Instruction,
        instruction: Instruction,
    ) {
        unsafe {
            instrlist_replace(self.raw, old_instruction.raw, instruction.raw);
        }
    }

    pub fn remove(
        &mut self,
        instruction: &Instruction,
    ) {
        unsafe {
            instrlist_remove(self.raw, instruction.raw);
        }
    }

    pub fn insert_clean_call(
        &mut self,
        anchor: &Instruction,
        func: extern "C" fn() -> (),
        save_fpstate: bool,
    ) {
        unsafe {
            dr_insert_clean_call(
                self.context,
                self.raw,
                anchor.raw,
                func as *mut core::ffi::c_void,
                save_fpstate as _,
                0,
            )
        }
    }

    pub fn insert_call_instrumentation(
        &mut self,
        anchor: &Instruction,
        func: extern "C" fn (usize, usize) -> (),
    ) {
        unsafe {
            dr_insert_call_instrumentation(
                self.context,
                self.raw,
                anchor.raw,
                func as _,
            )
        }
    }

    pub fn insert_mbr_instrumentation(
        &mut self,
        anchor: &Instruction,
        func: extern "C" fn (usize, usize) -> (),
        spill_slot: dr_spill_slot_t,
    ) {
        unsafe {
            dr_insert_mbr_instrumentation(
                self.context,
                self.raw,
                anchor.raw,
                func as _,
                spill_slot,
            )
        }
    }

}

impl Drop for InstructionList {
    fn drop(&mut self) {
        unsafe {
            instrlist_destroy(self.context, self.raw);
        }
    }
}

impl Clone for InstructionList {
    fn clone(&self) -> Self {
        let raw = unsafe {
            instrlist_clone(self.context, self.raw)
        };

        Self {
            context: self.context,
            raw,
        }
    }
}
