use atomic::{Atomic, Ordering};
use core::alloc::{GlobalAlloc, Layout};
use dynamorio_sys::*;

pub static MALLOC: Atomic<Option<extern "C" fn(usize) -> *mut core::ffi::c_void>> = Atomic::new(None);
pub static FREE: Atomic<Option<extern "C" fn(*mut core::ffi::c_void)>> = Atomic::new(None);

pub struct Allocator;

unsafe impl GlobalAlloc for Allocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let use_app_mem = dr_app_running_under_dynamorio() != 0;

        let ptr = if use_app_mem {
            if let Some(malloc) = MALLOC.load(Ordering::Relaxed) {
                malloc(layout.size())
            } else {
                core::ptr::null_mut()
            }
        } else {
            __wrap_malloc(layout.size())
        };

        if ptr.is_null() {
            panic!("error: failed to allocate memory");
        }

        ptr as *mut u8
    }

    unsafe fn dealloc(&self, ptr: *mut u8, _layout: Layout) {
        let use_app_mem = dr_app_running_under_dynamorio() != 0;

        if use_app_mem {
            if let Some(free) = FREE.load(Ordering::Relaxed) {
                free(ptr as *mut core::ffi::c_void);
            }
        } else {
            __wrap_free(ptr as *mut core::ffi::c_void);
        }
    }
}

#[global_allocator]
static ALLOCATOR: Allocator = Allocator;
