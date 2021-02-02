#[macro_use]
extern crate log;
use pnet::packet::tcp::TcpFlags;
use std::collections::HashMap;
use std::env;
use std::fs;
use std::net::Ipv4Addr;
use std::process;

/// Usage:
/// $ cargo run 127.0.0.1 sS

const SYN_METHOD: &str = "sS";
const FIN_METHOD: &str = "sF";
const XMAX_METHOD: &str = "sX";
const NULL_METHOD: &str = "sN";

#[derive(Debug)]
struct PacketInfo {
    my_ipaddr: Ipv4Addr,
    target_ipaddr: Ipv4Addr,
    my_port: u16,
    maximum_port: u16,
    scan_type: ScanType,
}

#[derive(Copy, Clone, Debug)]
enum ScanType {
    Syn = TcpFlags::SYN as isize,
    Fin = TcpFlags::FIN as isize,
    Xmas = (TcpFlags::FIN | TcpFlags::URG | TcpFlags::PSH) as isize,
    Null = 0,
}

fn main() {
    env::set_var("RUST_LOG", "debug");
    env_logger::init();
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        error!("Bad number of arguments. [ipaddr] [scantype]");
        process::exit(1);
    }

    let packet_info = {
        let contents = fs::read_to_string(".env").expect("File .env read error");
        let map: HashMap<_, _> = contents
            .lines()
            .filter_map(|line| {
                let tokens = line.split("=").map(str::trim).collect::<Vec<_>>();
                match tokens.len() {
                    2 => Some((tokens[0], tokens[1])),
                    _ => None,
                }
            })
            .collect();

        PacketInfo {
            my_ipaddr: map["MY_IPADDR"].parse().expect("invalid ipaddr"),
            target_ipaddr: args[1].parse().expect("invalid target ipaddr"),
            my_port: map["MY_PORT"].parse().expect("invalid port number"),
            maximum_port: map["MAXIMUM_PORT_NUM"]
                .parse()
                .expect("invalid maximum port num"),
            scan_type: match args[2].as_str() {
                SYN_METHOD => ScanType::Syn,
                FIN_METHOD => ScanType::Fin,
                XMAX_METHOD => ScanType::Xmas,
                NULL_METHOD => ScanType::Null,
                _ => {
                    error!(
                        "Undefined scan method, only accept [{}|{}|{}|{}].",
                        SYN_METHOD, FIN_METHOD, XMAX_METHOD, NULL_METHOD
                    );
                    process::exit(1);
                }
            },
        }
    };

    println!("Packet info: {:?}", packet_info);
}
