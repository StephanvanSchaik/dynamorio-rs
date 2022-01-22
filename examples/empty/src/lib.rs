#![no_std]

use drstd::println;
use drstd::sync::{Arc, Mutex, Once};
use dynamorio_rs::*;

struct Client {
    registered_exit_handler: Option<RegisteredExitHandler<Self>>,
}

static CLIENT: Once<Arc<Mutex<Client>>> = Once::new();

impl ExitHandler for Client {
    fn exit(&mut self) {
        println!("Bye bye");
    }
}

#[no_mangle]
fn client_main(_id: ClientId, _args: &[&str]) {
    set_client_name("empty", "https://github.com/StephanvanSchaik/dynamorio-rs/issues");

    CLIENT.call_once(|| {
        let client = Arc::new(Mutex::new(Client {
            registered_exit_handler: None,
        }));

        let registered_exit_handler = register_exit_handler(&client);

        if let Ok(mut client) = client.lock() {
            client.registered_exit_handler = Some(registered_exit_handler);
        }

        client
    });

    println!("Hello, world!");
}
