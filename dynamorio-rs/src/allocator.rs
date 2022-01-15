use core::alloc::{GlobalAlloc, Layout};
use dynamorio_sys::*;

pub struct Allocator;

unsafe impl GlobalAlloc for Allocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let ptr = dr_global_alloc(layout.size());

        if ptr.is_null() {
            panic!("error: failed to allocate memory");
        }

        ptr as *mut u8
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        dr_global_free(ptr as *mut core::ffi::c_void, layout.size());
    }
}

#[global_allocator]
static ALLOCATOR: Allocator = Allocator;
