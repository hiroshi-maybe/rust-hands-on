use log::error;
use pnet::datalink;
use std::env;

fn main() {
    env::set_var("RUST_LOG", "debug");
    env_logger::init();
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        error!("No target interface name found");
        std::process::exit(1);
    }
    let interface_name = &args[1];

    let interfaces = datalink::interfaces();
    let interface = interfaces.into_iter().find(|iface| iface.name == *interface_name).expect("Failed to get interface");
}
