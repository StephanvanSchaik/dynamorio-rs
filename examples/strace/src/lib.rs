use dynamorio_rs::*;
use syscalls::Sysno;

fn before_syscall_event(context: &mut BeforeSyscallContext, sysnum: i32) -> bool {
    let sysno = Sysno::from(sysnum);

    println!("{}", sysno.name());
    true
}

fn event_exit() {
}

#[no_mangle]
fn client_main(_id: ClientId, _args: &[&str]) {
    let manager = Manager::new();
    set_client_name("strace", "https://github.com/StephanvanSchaik/dynamorio-rs/issues");

    manager.register_before_syscall_event(before_syscall_event);
    register_exit_event(event_exit);

    println!("Hello, world!");
}
