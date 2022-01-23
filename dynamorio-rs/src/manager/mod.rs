pub mod module;
pub mod syscall;
pub mod thread;

use atomic::{Atomic, Ordering};
use crate::{Context, Instruction, InstructionList};
use dynamorio_sys::*;

pub use module::*;
pub use syscall::*;
pub use thread::*;

pub use dynamorio_sys::dr_emit_flags_t;

static BB_ANALYSIS_HANDLER: Atomic<Option<fn(&mut Context, &InstructionList, bool, bool) -> dr_emit_flags_t>> = Atomic::new(None);
static BB_INSTRUMENTATION_HANDLER: Atomic<Option<fn(&mut Context, &mut InstructionList, &Instruction, bool, bool) -> dr_emit_flags_t>> = Atomic::new(None);

extern "C" fn bb_analysis_event(
    context: *mut core::ffi::c_void,
    _tag: *mut core::ffi::c_void,
    bb: *mut instrlist_t,
    for_trace: i8,
    translating: i8,
    _user_data: *mut *mut core::ffi::c_void,
) -> dr_emit_flags_t {
    let bb = InstructionList::from_raw(context, bb);
    let for_trace = for_trace != 0;
    let translating = translating != 0;
    let mut context = Context::from_raw(context);
    let mut flags = dr_emit_flags_t::DR_EMIT_DEFAULT;

    if let Some(handler) = BB_ANALYSIS_HANDLER.load(Ordering::Relaxed) {
        flags = handler(&mut context, &bb, for_trace, translating);
    }

    core::mem::forget(context);
    core::mem::forget(bb);

    flags
}

extern "C" fn bb_instrumentation_event(
    context: *mut core::ffi::c_void,
    _tag: *mut core::ffi::c_void,
    bb: *mut instrlist_t,
    instr: *mut instr_t,
    for_trace: i8,
    translating: i8,
    _user_data: *mut core::ffi::c_void,
) -> dr_emit_flags_t {
    let mut bb = InstructionList::from_raw(context, bb);
    let instr = Instruction::from_raw(context, instr);
    let for_trace = for_trace != 0;
    let translating = translating != 0;
    let mut context = Context::from_raw(context);
    let mut flags = dr_emit_flags_t::DR_EMIT_DEFAULT;

    if let Some(handler) = BB_INSTRUMENTATION_HANDLER.load(Ordering::Relaxed) {
        flags = handler(&mut context, &mut bb, &instr, for_trace, translating);
    }

    core::mem::forget(context);
    core::mem::forget(bb);
    core::mem::forget(instr);

    flags
}

pub struct Manager;

impl Manager {
    pub fn new() -> Self {
        unsafe {
            drmgr_init();
        }

        Self
    }

    pub fn register_bb_instrumentation_event(
        &self,
        analysis_func: Option<fn(&mut Context, &InstructionList, bool, bool) -> dr_emit_flags_t>,
        instrumentation_func: Option<fn(&mut Context, &mut InstructionList, &Instruction, bool, bool) -> dr_emit_flags_t>,
    ) {
        BB_ANALYSIS_HANDLER.store(analysis_func, Ordering::Relaxed);
        BB_INSTRUMENTATION_HANDLER.store(instrumentation_func, Ordering::Relaxed);

        unsafe {
            drmgr_register_bb_instrumentation_event(
                match analysis_func {
                    Some(_) => Some(bb_analysis_event),
                    _ => None,
                },
                match instrumentation_func {
                    Some(_) => Some(bb_instrumentation_event),
                    _ => None,
                },
                core::ptr::null_mut(),
            );
        }
    }
}
