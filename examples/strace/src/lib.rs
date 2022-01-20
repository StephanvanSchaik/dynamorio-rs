#![no_std]

extern crate alloc;

use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec;
use alloc::vec::Vec;
use drstd::*;
use drstd::sync::{Arc, Mutex, Once};
use dynamorio_rs::*;
use syscalls::{Sysno, Argument};

struct Client {
    before_syscall_token: Option<dynamorio_rs::manager::BeforeSyscallToken<Self>>,
    after_syscall_token: Option<dynamorio_rs::manager::AfterSyscallToken<Self>>,
    sysno: Option<Sysno>,
    arguments: Vec<u64>,
}

static CLIENT: Once<Arc<Mutex<Client>>> = Once::new();

impl dynamorio_rs::manager::BeforeSyscall for Client {
    fn before_syscall(&mut self, context: &mut BeforeSyscallContext, sysnum: i32) -> bool {
        let sysno = Sysno::from(sysnum);

        self.sysno = Some(sysno);
        self.arguments = sysno.arguments()
            .iter()
            .enumerate()
            .map(|(i, _argument)| unsafe { context.param(i) })
            .collect::<Vec<u64>>();

        true
    }
}

impl dynamorio_rs::manager::AfterSyscall for Client {
    fn after_syscall(&mut self, context: &mut AfterSyscallContext, _sysnum: i32) {
        let sysno = match self.sysno.take() {
            Some(sysno) => sysno,
            _ => return,
        };

        let arguments = self.arguments
            .iter()
            .map(|argument| format!("0x{argument:x}"))
            .collect::<Vec<String>>()
            .join(", ");

        println!("{}({}) = 0x{:x}", sysno.name(), arguments, context.get_result());
    }
}

fn filter_syscall_event(context: &mut Context, _sysnum: i32) -> bool {
    true
}

#[no_mangle]
fn client_main(_id: ClientId, _args: &[&str]) {
    let manager = Manager::new();
    set_client_name("strace", "https://github.com/StephanvanSchaik/dynamorio-rs/issues");

    CLIENT.call_once(|| {
        let client = Arc::new(Mutex::new(Client {
            before_syscall_token: None,
            after_syscall_token: None,
            sysno: None,
            arguments: vec![],
        }));

        let before_syscall_token = manager.register_before_syscall_event(&client);
        let after_syscall_token = manager.register_after_syscall_event(&client);

        if let Ok(mut client) = client.lock() {
            client.before_syscall_token = Some(before_syscall_token);
            client.after_syscall_token = Some(after_syscall_token);
        }

        client
    });

    register_filter_syscall_event(filter_syscall_event);
}
