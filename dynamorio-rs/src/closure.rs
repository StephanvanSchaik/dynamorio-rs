use dynamorio_sys::*;

pub struct Closure {
    inner: *mut core::ffi::c_void,
    size: usize,
}

impl Closure {
    fn lookup_code(args: usize) -> &'static [u8] {
        match args {
            0 => &[
                0x48, 0x8b, 0x3d, 0xe9, 0xff, 0xff, 0xff,
                0xff, 0x25, 0xeb, 0xff, 0xff, 0xff,
            ],
            1 => &[
                0x48, 0x8b, 0x35, 0xe9, 0xff, 0xff, 0xff,
                0xff, 0x25, 0xeb, 0xff, 0xff, 0xff,
            ],
            2 => &[
                0x48, 0x8b, 0x15, 0xe9, 0xff, 0xff, 0xff,
                0xff, 0x25, 0xeb, 0xff, 0xff, 0xff,
            ],
            3 => &[
                0x48, 0x8b, 0x0d, 0xe9, 0xff, 0xff, 0xff,
                0xff, 0x25, 0xeb, 0xff, 0xff, 0xff,
            ],
            4 => &[
                0x4c, 0x8b, 0x05, 0xe9, 0xff, 0xff, 0xff,
                0xff, 0x25, 0xeb, 0xff, 0xff, 0xff
            ],
            5 => &[
                0x4c, 0x8b, 0x0d, 0xe9, 0xff, 0xff, 0xff,
                0xff, 0x25, 0xeb, 0xff, 0xff, 0xff,
            ],
            6 => &[
                0xff, 0x35, 0xea, 0xff, 0xff, 0xff,
                0xff, 0x15, 0xec, 0xff, 0xff, 0xff,
                0x48, 0x83, 0xc4, 0x08,
                0xc3,
            ],
            7 => &[
                0x48, 0x83, 0xec, 0x08,
                0xff, 0x35, 0xe6, 0xff, 0xff, 0xff,
                0xff, 0x74, 0x24, 0x18,
                0xff, 0x15, 0xe4, 0xff, 0xff, 0xff,
                0x48, 0x83, 0xc4, 0x18,
                0xc3,
            ],
            8 => &[
                0xff, 0x35, 0xea, 0xff, 0xff, 0xff,
                0xff, 0x74, 0x24, 0x18,
                0xff, 0x74, 0x24, 0x18,
                0xff, 0x15, 0xe4, 0xff, 0xff, 0xff,
                0x48, 0x83, 0xc4, 0x18,
                0xc3,
            ],
            _ => panic!("argument count not supported"),
        }
    }

    pub fn new(
        args: usize,
        callback: fn(),
        user_data: *mut core::ffi::c_void,
    ) -> Self {
        let code = Self::lookup_code(args);
        let size = 2 * core::mem::size_of::<u64>() + code.len();

        let inner = unsafe {
            dr_nonheap_alloc(size, DR_MEMPROT_READ | DR_MEMPROT_WRITE | DR_MEMPROT_EXEC)
        };

        let storage: &mut [u64] = unsafe {
            core::slice::from_raw_parts_mut(inner as *mut u64, 2)
        };

        storage[0] = user_data as u64;
        storage[1] = callback as u64;

        let storage: &mut [u8] = unsafe {
            core::slice::from_raw_parts_mut(
                (inner as *mut u8).add(2 * core::mem::size_of::<u64>()),
                code.len(),
            )
        };

        storage.copy_from_slice(code);

        Self {
            inner,
            size,
        }
    }

    pub fn code(&self) -> *mut core::ffi::c_void {
        unsafe {
            (self.inner as *mut u8)
                .add(2 * core::mem::size_of::<u64>())
                as *mut core::ffi::c_void
        }
    }
}

impl Drop for Closure {
    fn drop(&mut self) {
        unsafe {
            dr_nonheap_free(self.inner, self.size);
        }
    }
}
