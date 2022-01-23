use atomic::{Atomic, Ordering};
use crate::{AfterSyscallContext, BeforeSyscallContext, Context, Instruction, InstructionList, ModuleData};
use crate::closure::Closure;
use drstd::sync::{Arc, Mutex};
use dynamorio_sys::*;

pub use dynamorio_sys::dr_emit_flags_t;

static BB_ANALYSIS_HANDLER: Atomic<Option<fn(&mut Context, &InstructionList, bool, bool) -> dr_emit_flags_t>> = Atomic::new(None);
static BB_INSTRUMENTATION_HANDLER: Atomic<Option<fn(&mut Context, &mut InstructionList, &Instruction, bool, bool) -> dr_emit_flags_t>> = Atomic::new(None);

pub trait SyscallHandler {
    fn filter_syscall(&mut self, context: &mut Context, sysno: i32) -> bool;
    fn before_syscall(&mut self, context: &mut BeforeSyscallContext, sysno: i32) -> bool;
    fn after_syscall(&mut self, context: &mut AfterSyscallContext, sysno: i32);
}

pub struct RegisteredSyscallHandler<T: SyscallHandler> {
    _handler: Arc<Mutex<T>>,
    closure: Closure,
}

unsafe impl<T: SyscallHandler> Send for RegisteredSyscallHandler<T> {}
unsafe impl<T: SyscallHandler> Sync for RegisteredSyscallHandler<T> {}

impl<T: SyscallHandler> Drop for RegisteredSyscallHandler<T> {
    fn drop(&mut self) {
        let func: extern "C" fn(*mut core::ffi::c_void, i32) -> i8 = unsafe {
            core::mem::transmute(self.closure.code())
        };

        unsafe {
            dr_unregister_filter_syscall_event(
                Some(func),
            );
            drmgr_unregister_pre_syscall_event_user_data(
                Some(before_syscall_event::<T>),
            );
            drmgr_unregister_post_syscall_event_user_data(
                Some(after_syscall_event::<T>),
            );
        }
    }
}

extern "C" fn filter_syscall_event<T: SyscallHandler>(
    context: *mut core::ffi::c_void,
    sysnum: i32,
    handler: &Mutex<T>,
) -> i8 {
    let mut context = Context::from_raw(context);
    let mut result = false;

    if let Ok(mut handler) = handler.lock() {
        result = handler.filter_syscall(&mut context, sysnum);
    }

    result as i8
}

extern "C" fn before_syscall_event<T: SyscallHandler>(
    context: *mut core::ffi::c_void,
    sysnum: i32,
    user_data: *mut core::ffi::c_void,
) -> i8 {
    let mut context = BeforeSyscallContext::from_raw(context);
    let handler = unsafe { &*(user_data as *mut Mutex<T>) };
    let mut result = 0;

    if let Ok(mut handler) = handler.lock() {
        result = handler.before_syscall(&mut context, sysnum) as i8;
    }

    result
}

extern "C" fn after_syscall_event<T: SyscallHandler>(
    context: *mut core::ffi::c_void,
    sysnum: i32,
    user_data: *mut core::ffi::c_void,
) {
    let mut context = AfterSyscallContext::from_raw(context);
    let handler = unsafe { &*(user_data as *mut Mutex<T>) };

    if let Ok(mut handler) = handler.lock() {
        handler.after_syscall(&mut context, sysnum);
    }
}

pub trait ModuleHandler {
    fn load_module(&mut self, context: &mut Context, module: &ModuleData, loaded: bool);
    fn unload_module(&mut self, context: &mut Context, module: &ModuleData);
}

pub struct RegisteredModuleHandler<T: ModuleHandler> {
    _handler: Arc<Mutex<T>>,
}

impl<T: ModuleHandler> Drop for RegisteredModuleHandler<T> {
    fn drop(&mut self) {
        unsafe {
            drmgr_unregister_module_load_event_user_data(
                Some(module_load_event::<T>),
            );
        }

        unsafe {
            drmgr_unregister_module_unload_event_user_data(
                Some(module_unload_event::<T>),
            );
        }
    }
}

extern "C" fn module_load_event<T: ModuleHandler>(
    context: *mut core::ffi::c_void,
    module: *const module_data_t,
    loaded: i8,
    user_data: *mut core::ffi::c_void,
) {
    let module = ModuleData::from_raw(module as _);
    let loaded = loaded != 0;
    let mut context = Context::from_raw(context);
    let handler = unsafe { &*(user_data as *mut Mutex<T>) };

    if let Ok(mut handler) = handler.lock() {
        handler.load_module(&mut context, &module, loaded);
    }

    core::mem::forget(context);
    core::mem::forget(module);
}

extern "C" fn module_unload_event<T: ModuleHandler>(
    context: *mut core::ffi::c_void,
    module: *const module_data_t,
    user_data: *mut core::ffi::c_void,
) {
    let module = ModuleData::from_raw(module as _);
    let mut context = Context::from_raw(context);
    let handler = unsafe { &*(user_data as *mut Mutex<T>) };

    if let Ok(mut handler) = handler.lock() {
        handler.unload_module(&mut context, &module);
    }

    core::mem::forget(context);
    core::mem::forget(module);
}

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

extern "C" fn thread_init_event(
    context: *mut core::ffi::c_void,
    user_data: *mut core::ffi::c_void,
) {
    let mut context = Context::from_raw(context);
    let func = unsafe {
        core::mem::transmute::<*mut core::ffi::c_void, fn(&mut Context) -> ()>(user_data)
    };

    func(&mut context);
}

extern "C" fn thread_exit_event(
    context: *mut core::ffi::c_void,
    user_data: *mut core::ffi::c_void,
) {
    let mut context = Context::from_raw(context);
    let func = unsafe {
        core::mem::transmute::<*mut core::ffi::c_void, fn(&mut Context) -> ()>(user_data)
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

    pub fn register_syscall_handler<T: SyscallHandler>(
        &self,
        handler: &Arc<Mutex<T>>,
    ) -> RegisteredSyscallHandler<T> {
        let closure = Closure::new(
            2,
            unsafe {
                core::mem::transmute(filter_syscall_event::<T> as extern "C" fn(_, _, _) -> _)
            },
            Arc::as_ptr(&handler) as *mut core::ffi::c_void,
        );

        let func: extern "C" fn(*mut core::ffi::c_void, i32) -> i8 = unsafe {
            core::mem::transmute(closure.code())
        };

        unsafe {
            dr_register_filter_syscall_event(Some(func));
            drmgr_register_pre_syscall_event_user_data(
                Some(before_syscall_event::<T>),
                core::ptr::null_mut(),
                Arc::as_ptr(&handler) as *mut core::ffi::c_void,
            );
            drmgr_register_post_syscall_event_user_data(
                Some(after_syscall_event::<T>),
                core::ptr::null_mut(),
                Arc::as_ptr(&handler) as *mut core::ffi::c_void,
            );
        }

        RegisteredSyscallHandler {
            _handler: Arc::clone(&handler),
            closure,
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
                core::ptr::null_mut(),
            );
        }
    }

    pub fn register_module_handler<T: ModuleHandler>(
        &self,
        handler: &Arc<Mutex<T>>,
    ) -> RegisteredModuleHandler<T> {
        unsafe {
            drmgr_register_module_load_event_user_data(
                Some(module_load_event::<T>),
                core::ptr::null_mut(),
                Arc::as_ptr(&handler) as *mut core::ffi::c_void,
            );
        }

        unsafe {
            drmgr_register_module_unload_event_user_data(
                Some(module_unload_event::<T>),
                core::ptr::null_mut(),
                Arc::as_ptr(&handler) as *mut core::ffi::c_void,
            );
        }

        RegisteredModuleHandler {
            _handler: Arc::clone(&handler),
        }
    }

    pub fn register_thread_init_event(
        &self,
        func: fn(&mut Context) -> (),
    ) {
        unsafe {
            drmgr_register_thread_init_event_user_data(
                Some(thread_init_event),
                core::ptr::null_mut(),
                func as *mut core::ffi::c_void,
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
                core::ptr::null_mut(),
                func as *mut core::ffi::c_void,
            );
        }
    }
}
