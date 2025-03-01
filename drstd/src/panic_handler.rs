use crate::eprintln;
use dynamorio_sys::dr_abort_with_code;

#[panic_handler]
fn panic(panic_info: &core::panic::PanicInfo<'_>) -> ! {
    if let Some(location) = panic_info.location() {
        let file = location.file();
        let line = location.line();
        let col = location.column();

        eprintln!("client panicked at {file}:{line}:{col}:");
    }

    let message = panic_info.message();

    eprintln!("{message}");

    unsafe {
        dr_abort_with_code(-1);
    }

    loop {}
}
