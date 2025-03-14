#[cfg(feature = "io")]
pub use no_std_io::io::{Cursor, Read, Seek, SeekFrom, Write};
use dynamorio_sys::*;

#[doc(hidden)]
pub fn _print(args: core::fmt::Arguments) {
    let s = alloc::format!("{}\0", args);

    // Print the string.
    unsafe {
        dr_printf(c"%s".as_ptr() as *const _, s.as_ptr());
    }
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::io::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

#[doc(hidden)]
pub fn _eprint(args: core::fmt::Arguments) {
    let s = alloc::format!("{}\0", args);

    // Print the string.
    unsafe {
        dr_fprintf(dr_stderr(), c"%s".as_ptr() as *const _, s.as_ptr());
    }
}

#[macro_export]
macro_rules! eprint {
    ($($arg:tt)*) => ($crate::io::_eprint(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! eprintln {
    () => ($crate::eprint!("\n"));
    ($($arg:tt)*) => ($crate::eprint!("{}\n", format_args!($($arg)*)));
}
