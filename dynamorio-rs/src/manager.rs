use atomic::{Atomic, Ordering};
use crate::{Context, Instruction, InstructionList, ModuleData};
use dynamorio_sys::*;

pub use dynamorio_sys::dr_emit_flags_t;

static BB_ANALYSIS_HANDLER: Atomic<Option<fn(&mut Context, &InstructionList, bool, bool) -> dr_emit_flags_t>> = Atomic::new(None);
static BB_INSTRUMENTATION_HANDLER: Atomic<Option<fn(&mut Context, &mut InstructionList, &Instruction, bool, bool) -> dr_emit_flags_t>> = Atomic::new(None);

extern "C" fn before_syscall_event(
    context: *mut std::ffi::c_void,
    sysnum: i32,
    user_data: *mut std::ffi::c_void,
) -> i8 {
    let mut context = Context::from_raw(context);
    let func = unsafe {
        std::mem::transmute::<*mut std::ffi::c_void, fn(&mut Context, i32) -> bool>(user_data)
    };

    let result = func(&mut context, sysnum) as i8;

    result
}

extern "C" fn after_syscall_event(
    context: *mut std::ffi::c_void,
    sysnum: i32,
    user_data: *mut std::ffi::c_void,
) {
    let mut context = Context::from_raw(context);
    let func = unsafe {
        std::mem::transmute::<*mut std::ffi::c_void, fn(&mut Context, i32) -> ()>(user_data)
    };

    func(&mut context, sysnum);
}

extern "C" fn module_load_event(
    context: *mut std::ffi::c_void,
    module: *const module_data_t,
    loaded: std::os::raw::c_char,
    user_data: *mut std::ffi::c_void,
) {
    let module = ModuleData::from_raw(module as _);
    let loaded = loaded != 0;
    let mut context = Context::from_raw(context);
    let func = unsafe {
        std::mem::transmute::<*mut std::ffi::c_void, fn(&mut Context, &ModuleData, bool) -> ()>(user_data)
    };

    func(&mut context, &module, loaded);

    std::mem::forget(context);
    std::mem::forget(module);
}

extern "C" fn module_unload_event(
    context: *mut std::ffi::c_void,
    module: *const module_data_t,
    user_data: *mut std::ffi::c_void,
) {
    let module = ModuleData::from_raw(module as _);
    let mut context = Context::from_raw(context);
    let func = unsafe {
        std::mem::transmute::<*mut std::ffi::c_void, fn(&mut Context, &ModuleData) -> ()>(user_data)
    };

    func(&mut context, &module);

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

extern "C" fn thread_init_event(
    context: *mut std::ffi::c_void,
    user_data: *mut std::ffi::c_void,
) {
    let mut context = Context::from_raw(context);
    let func = unsafe {
        std::mem::transmute::<*mut std::ffi::c_void, fn(&mut Context) -> ()>(user_data)
    };

    func(&mut context);
}

extern "C" fn thread_exit_event(
    context: *mut std::ffi::c_void,
    user_data: *mut std::ffi::c_void,
) {
    let mut context = Context::from_raw(context);
    let func = unsafe {
        std::mem::transmute::<*mut std::ffi::c_void, fn(&mut Context) -> ()>(user_data)
    };

    func(&mut context);
}

pub struct Manager;

impl Manager {
    pub fn new() -> Self {
        unsafe {
            drmgr_init();
        }

        Self
    }

    #[cfg(target_os = "windows")]
    pub fn decode_sysnum_from_wrapper(
        &self,
        entry: usize,
    ) -> Option<u32> {
        let result = unsafe {
            drmgr_decode_sysnum_from_wrapper(entry)
        };

        if result < 0 {
            return None;
        }

        Some(result as u32)
    }

    pub fn register_before_syscall_event(
        &self,
        func: fn(&mut Context, i32) -> bool,
    ) {
        unsafe {
            drmgr_register_pre_syscall_event_user_data(
                Some(before_syscall_event),
                std::ptr::null_mut(),
                func as *mut std::ffi::c_void,
            );
        }
    }

    pub fn register_after_syscall_event(
        &self,
        func: fn(&mut Context, i32) -> (),
    ) {
        unsafe {
            drmgr_register_post_syscall_event_user_data(
                Some(after_syscall_event),
                std::ptr::null_mut(),
                func as *mut std::ffi::c_void,
            );
        }
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
        unsafe {
            drmgr_register_module_load_event_user_data(
                Some(module_load_event),
                std::ptr::null_mut(),
                func as *mut std::ffi::c_void,
            );
        }
    }

    pub fn register_module_unload_event(
        &self,
        func: fn(&mut Context, &ModuleData) -> (),
    ) {
        unsafe {
            drmgr_register_module_unload_event_user_data(
                Some(module_unload_event),
                std::ptr::null_mut(),
                func as *mut std::ffi::c_void,
            );
        }
    }

    pub fn register_thread_init_event(
        &self,
        func: fn(&mut Context) -> (),
    ) {
        unsafe {
            drmgr_register_thread_init_event_user_data(
                Some(thread_init_event),
                std::ptr::null_mut(),
                func as *mut std::ffi::c_void,
            );
        }
    }

    pub fn register_thread_exit_event(
        &self,
        func: fn(&mut Context) -> (),
    ) {
        unsafe {
            drmgr_register_thread_exit_event_user_data(
                Some(thread_exit_event),
                std::ptr::null_mut(),
                func as *mut std::ffi::c_void,
            );
        }
    }
}