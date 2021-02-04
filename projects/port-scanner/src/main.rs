#[macro_use]
extern crate log;
use pnet::packet::ip::IpNextHeaderProtocols;
use pnet::packet::tcp::{self, MutableTcpPacket, TcpFlags};
use pnet::transport::{self, TransportChannelType, TransportProtocol};
use std::collections::HashMap;
use std::env;
use std::fs;
use std::net::{self, Ipv4Addr};
use std::process;
use std::thread;
use std::time;

/// Usage:
/// $ cargo run 127.0.0.1 sS

const TCP_SIZE: usize = 20;
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

    let (mut ts, mut tr) = transport::transport_channel(
        1024,
        TransportChannelType::Layer4(TransportProtocol::Ipv4(IpNextHeaderProtocols::Tcp)),
    )
    .expect("Failed to open channel.");

    send_packets(&mut ts, &packet_info);
}

fn send_packets(ts: &mut transport::TransportSender, packet_info: &PacketInfo) {
    let mut packet = build_packet(packet_info);
    for port in 1..packet_info.maximum_port {
        let mut tcp_header = tcp::MutableTcpPacket::new(&mut packet).unwrap();
        reregister_destination_port(port, &mut tcp_header, packet_info);
        thread::sleep(time::Duration::from_millis(5));
        ts.send_to(tcp_header, net::IpAddr::V4(packet_info.target_ipaddr))
            .expect("failed to send a packet");
    }
}

fn reregister_destination_port(
    target: u16,
    tcp_header: &mut MutableTcpPacket,
    packet_info: &PacketInfo,
) {
    tcp_header.set_destination(target);
    set_checksum(tcp_header, packet_info);
}

fn build_packet(packet_info: &PacketInfo) -> [u8; TCP_SIZE] {
    let mut tcp_buffer = [0u8; TCP_SIZE];
    let mut tcp_header = MutableTcpPacket::new(&mut tcp_buffer[..]).unwrap();
    tcp_header.set_source(packet_info.my_port);
    tcp_header.set_data_offset(5);
    tcp_header.set_flags(packet_info.scan_type as u16);
    set_checksum(&mut tcp_header, packet_info);

    tcp_buffer
}

fn set_checksum(tcp_header: &mut MutableTcpPacket, packet_info: &PacketInfo) {
    let checksum = tcp::ipv4_checksum(
        &tcp_header.to_immutable(),
        &packet_info.my_ipaddr,
        &packet_info.target_ipaddr,
    );
    tcp_header.set_checksum(checksum);
}
