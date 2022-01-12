use atomic::{Atomic, Ordering};
use crate::{Context, Instruction, InstructionList, ModuleData};
use dynamorio_sys::*;

pub use dynamorio_sys::dr_emit_flags_t;

static MODULE_LOAD_HANDLER: Atomic<Option<fn(&mut Context, &ModuleData, bool) -> ()>> = Atomic::new(None);
static BB_ANALYSIS_HANDLER: Atomic<Option<fn(&mut Context, &InstructionList, bool, bool) -> dr_emit_flags_t>> = Atomic::new(None);
static BB_INSTRUMENTATION_HANDLER: Atomic<Option<fn(&mut Context, &mut InstructionList, &Instruction, bool, bool) -> dr_emit_flags_t>> = Atomic::new(None);

extern "C" fn module_load_event(
    context: *mut std::ffi::c_void,
    module: *const module_data_t,
    loaded: std::os::raw::c_char,
) {
    let module = ModuleData::from_raw(module as _);
    let loaded = loaded != 0;
    let mut context = Context::from_raw(context);

    if let Some(handler) = MODULE_LOAD_HANDLER.load(Ordering::Relaxed) {
        handler(&mut context, &module, loaded);
    }

    std::mem::forget(context);
    std::mem::forget(module);
}

extern "C" fn bb_analysis_event(
    context: *mut std::ffi::c_void,
    tag: *mut std::ffi::c_void,
    bb: *mut instrlist_t,
    for_trace: i8,
    translating: i8,
    user_data: *mut *mut std::ffi::c_void,
) -> dr_emit_flags_t {
    let bb = InstructionList::from_raw(context, bb);
    let for_trace = for_trace != 0;
    let translating = translating != 0;
    let mut context = Context::from_raw(context);
    let mut flags = dr_emit_flags_t::DR_EMIT_DEFAULT;

    if let Some(handler) = BB_ANALYSIS_HANDLER.load(Ordering::Relaxed) {
        flags = handler(&mut context, &bb, for_trace, translating);
    }

    std::mem::forget(context);
    std::mem::forget(bb);

    flags
}

extern "C" fn bb_instrumentation_event(
    context: *mut std::ffi::c_void,
    tag: *mut std::ffi::c_void,
    bb: *mut instrlist_t,
    instr: *mut instr_t,
    for_trace: i8,
    translating: i8,
    user_data: *mut std::ffi::c_void,
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

    std::mem::forget(context);
    std::mem::forget(bb);
    std::mem::forget(instr);

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
                std::ptr::null_mut(),
            );
        }
    }

    pub fn register_module_load_event(
        &self,
        func: fn(&mut Context, &ModuleData, bool) -> (),
    ) {
        MODULE_LOAD_HANDLER.store(Some(func), Ordering::Relaxed);

        unsafe {
            drmgr_register_module_load_event(Some(module_load_event));
        }
    }
}
