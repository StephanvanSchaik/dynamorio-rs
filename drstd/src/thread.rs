use core::time::Duration;
use dynamorio_sys::*;

pub fn sleep(dur: Duration) {
    unsafe {
        dr_sleep(dur.as_millis() as _);
    }
}

pub fn yield_now() {
    unsafe {
        dr_thread_yield();
    }
}
