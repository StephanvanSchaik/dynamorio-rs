pub mod basic_block;
pub mod module;
pub mod syscall;
pub mod thread;

use dynamorio_sys::*;

pub use basic_block::*;
pub use module::*;
pub use syscall::*;
pub use thread::*;

pub use dynamorio_sys::dr_emit_flags_t;

pub struct Manager;

impl Manager {
    pub fn new() -> Self {
        unsafe {
            drmgr_init();
        }

        Self
    }
}
