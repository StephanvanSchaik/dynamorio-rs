use crate::{Context, Instruction, InstructionList, Manager};
use crate::closure::Closure;
use dynamorio_sys::*;
use drstd::sync::{Arc, Mutex};

pub trait BasicBlockHandler {
    fn analyse(
        &mut self,
        context: &mut Context,
        bb: &InstructionList,
        for_trace: bool,
        translating: bool,
    ) -> dr_emit_flags_t;

    fn instrument(
        &mut self,
        context: &mut Context,
        bb: &mut InstructionList,
        instruction: &Instruction,
        for_trace: bool,
        translating: bool,
    ) -> dr_emit_flags_t;
}

extern "C" fn bb_analysis_event<T: BasicBlockHandler>(
    context: *mut core::ffi::c_void,
    _tag: *mut core::ffi::c_void,
    bb: *mut instrlist_t,
    for_trace: i8,
    translating: i8,
    _user_data: *mut *mut core::ffi::c_void,
    handler: &Mutex<T>,
) -> dr_emit_flags_t {
    let bb = InstructionList::from_raw(context, bb);
    let for_trace = for_trace != 0;
    let translating = translating != 0;
    let mut context = Context::from_raw(context);
    let mut flags = dr_emit_flags_t::DR_EMIT_DEFAULT;

    if let Ok(mut handler) = handler.lock() {
        flags = handler.analyse(&mut context, &bb, for_trace, translating);
    }

    core::mem::forget(context);
    core::mem::forget(bb);

    flags
}

extern "C" fn bb_instrumentation_event<T: BasicBlockHandler>(
    context: *mut core::ffi::c_void,
    _tag: *mut core::ffi::c_void,
    bb: *mut instrlist_t,
    instr: *mut instr_t,
    for_trace: i8,
    translating: i8,
    _user_data: *mut core::ffi::c_void,
    handler: &Arc<Mutex<T>>,
) -> dr_emit_flags_t {
    let mut bb = InstructionList::from_raw(context, bb);
    let instr = Instruction::from_raw(context, instr);
    let for_trace = for_trace != 0;
    let translating = translating != 0;
    let mut context = Context::from_raw(context);
    let mut flags = dr_emit_flags_t::DR_EMIT_DEFAULT;

    if let Ok(mut handler) = handler.lock() {
        flags = handler.instrument(&mut context, &mut bb, &instr, for_trace, translating);
    }

    core::mem::forget(context);
    core::mem::forget(bb);
    core::mem::forget(instr);

    flags
}

pub struct RegisteredBasicBlockHandler<T: BasicBlockHandler> {
    _handler: Arc<Mutex<T>>,
    bb_analysis_closure: Closure,
    _bb_instrumentation_closure: Closure,
}

unsafe impl<T: BasicBlockHandler> Send for RegisteredBasicBlockHandler<T> {}
unsafe impl<T: BasicBlockHandler> Sync for RegisteredBasicBlockHandler<T> {}

impl<T: BasicBlockHandler> Drop for RegisteredBasicBlockHandler<T> {
    fn drop(&mut self) {
        let bb_analysis_wrapper: extern "C" fn(*mut core::ffi::c_void, *mut core::ffi::c_void, *mut instrlist_t, i8, i8, *mut *mut core::ffi::c_void) -> dr_emit_flags_t = unsafe {
            core::mem::transmute(self.bb_analysis_closure.code())
        };

        unsafe {
            drmgr_unregister_bb_instrumentation_event(
                Some(bb_analysis_wrapper),
            );
        }
    }
}

impl Manager {
    pub fn instrument_basic_block<T: BasicBlockHandler>(handler: &Arc<Mutex<T>>) -> RegisteredBasicBlockHandler<T> {
        let bb_analysis_closure = Closure::new(
            6,
            unsafe {
                core::mem::transmute(bb_analysis_event::<T> as unsafe extern "C" fn(_, _, _, _, _, _, _) -> _)
            },
            Arc::as_ptr(&handler) as *mut core::ffi::c_void,
        );

        let bb_analysis_wrapper: extern "C" fn(*mut core::ffi::c_void, *mut core::ffi::c_void, *mut instrlist_t, i8, i8, *mut *mut core::ffi::c_void) -> dr_emit_flags_t = unsafe {
            core::mem::transmute(bb_analysis_closure.code())
        };

        let bb_instrumentation_closure = Closure::new(
            7,
            unsafe {
                core::mem::transmute(bb_instrumentation_event::<T> as unsafe extern "C" fn(_, _, _, _, _, _, _, _) -> _)
            },
            Arc::as_ptr(&handler) as *mut core::ffi::c_void,
        );

        let bb_instrumentation_wrapper: extern "C" fn(*mut core::ffi::c_void, *mut core::ffi::c_void, *mut instrlist_t, *mut instr_t, i8, i8, *mut core::ffi::c_void) -> dr_emit_flags_t = unsafe {
            core::mem::transmute(bb_instrumentation_closure.code())
        };

        unsafe {
            drmgr_register_bb_instrumentation_event(
                Some(bb_analysis_wrapper),
                Some(bb_instrumentation_wrapper),
                core::ptr::null_mut(),
            );
        }

        RegisteredBasicBlockHandler {
            _handler: Arc::clone(&handler),
            bb_analysis_closure,
            _bb_instrumentation_closure: bb_instrumentation_closure,
        }
    }
}
