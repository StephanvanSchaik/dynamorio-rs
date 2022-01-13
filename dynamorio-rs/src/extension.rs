use atomic::{Atomic, Ordering};
use dynamorio_sys::*;

static SOFT_KILLS_HANDLER: Atomic<Option<fn(process_id_t, i32) -> bool>> = Atomic::new(None);

extern "C" fn soft_kills_event(pid: process_id_t, exit_code: i32) -> i8 {
    let mut result = false;

    if let Some(handler) = SOFT_KILLS_HANDLER.load(Ordering::Relaxed) {
        result = handler(pid, exit_code) as bool;
    }

    result as i8
}

pub struct Extension;

impl Extension {
    pub fn new() -> Self {
        unsafe {
            drx_init();
        }

        Self
    }

    pub fn register_soft_kills(
        &self,
        func: fn(pid: process_id_t, exit_code: i32) -> bool,
    ) {
        SOFT_KILLS_HANDLER.store(Some(func), Ordering::Relaxed);

        unsafe {
            drx_register_soft_kills(Some(soft_kills_event));
        }
    }
}
