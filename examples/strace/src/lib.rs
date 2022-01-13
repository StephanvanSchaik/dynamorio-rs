use dynamorio_rs::*;
use syscalls::Sysno;

fn before_syscall_event(context: &mut BeforeSyscallContext, sysnum: i32) -> bool {
    let sysno = Sysno::from(sysnum);

    let arguments = sysno.arguments()
        .map(|arguments| {
            arguments
                .iter()
                .enumerate()
                .map(|(i, _argument)| {
                    let param = unsafe { context.param(i) };

                    format!("0x{:x}", param)
                })
                .collect::<Vec<String>>()
                .join(", ")
        })
        .unwrap_or("???".to_string());

    print!("{}({:?})", sysno.name(), arguments);

    true
}

fn after_syscall_event(context: &mut AfterSyscallContext, _sysnum: i32) {
    println!(" = 0x{:x}", context.get_result());
}

#[no_mangle]
fn client_main(_id: ClientId, _args: &[&str]) {
    let manager = Manager::new();
    set_client_name("strace", "https://github.com/StephanvanSchaik/dynamorio-rs/issues");

    manager.register_before_syscall_event(before_syscall_event);
    manager.register_after_syscall_event(after_syscall_event);

    // Make sure the system call definitions have been initialized.
    let sysno = Sysno::read;
    sysno.arguments();
}
