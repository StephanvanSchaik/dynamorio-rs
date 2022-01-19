#![no_std]

extern crate alloc;

use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use drstd::*;
use dynamorio_rs::*;
use syscalls::Sysno;

fn before_syscall_event(context: &mut BeforeSyscallContext, sysnum: i32) -> bool {
    let sysno = Sysno::from(sysnum);

    let arguments = sysno.arguments()
        .iter()
        .enumerate()
        .map(|(i, _argument)| {
            let param = unsafe { context.param(i) };

            format!("0x{:x}", param)
        })
        .collect::<Vec<String>>()
        .join(", ");

    print!("{}({:?})", sysno.name(), arguments);

    true
}

fn after_syscall_event(context: &mut AfterSyscallContext, _sysnum: i32) {
    println!(" = 0x{:x}", context.get_result());
}

fn filter_syscall_event(context: &mut Context, _sysnum: i32) -> bool {
    true
}

#[no_mangle]
fn client_main(_id: ClientId, _args: &[&str]) {
    let manager = Manager::new();
    set_client_name("strace", "https://github.com/StephanvanSchaik/dynamorio-rs/issues");

    manager.register_before_syscall_event(before_syscall_event);
    manager.register_after_syscall_event(after_syscall_event);
    register_filter_syscall_event(filter_syscall_event);

    // Make sure the system call definitions have been initialized.
    let sysno = Sysno::read;
    sysno.arguments();
}
