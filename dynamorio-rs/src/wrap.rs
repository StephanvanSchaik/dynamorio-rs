use dynamorio_sys::*;

pub struct Wrapper;

impl Wrapper {
    pub fn new() -> Self {
        unsafe {
            drwrap_init();
        }

        Self
    }
}
