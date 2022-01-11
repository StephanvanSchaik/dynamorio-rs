use dynamorio_rs::*;

fn event_exit() {
}

#[no_mangle]
fn client_main(id: ClientId, args: &[&str]) {
    set_client_name("empty", "https://github.com/StephanvanSchaik/dynamorio-rs/issues");

    register_exit_event(event_exit);

    println!("Hello, world!");
}
