#[macro_use]
extern crate log;
extern crate rayon;
use pnet::datalink;
use pnet::datalink::Channel::Ethernet;
use pnet::datalink::Config;
use pnet::datalink::DataLinkReceiver;
use pnet::packet::ethernet::{EtherTypes, EthernetPacket};
use pnet::packet::ip::IpNextHeaderProtocols;
use pnet::packet::ipv4::Ipv4Packet;
use pnet::packet::tcp::{self, MutableTcpPacket, TcpFlags, TcpPacket};
use pnet::packet::Packet;
use pnet::transport::{self, TransportChannelType, TransportProtocol};
use std::collections::HashMap;
use std::env;
use std::fs;
use std::net::{self, Ipv4Addr};
use std::process;
use std::thread;
use std::time;

/// Non-SYN method does not work for my router.
/// It always returns RST | ACK flag for non-SYN packet.
///
/// Usage:
/// $ cargo build
/// $ sudo ./target/debug/port-scanner 10.0.1.1 sS
/// $ sudo ./target/debug/port-scanner 10.0.1.1 sF (does not work)

const NIF_NAME: &str = "en0";
const TCP_SIZE: usize = 20;
const SYN_METHOD: &str = "sS";
const FIN_METHOD: &str = "sF";
const XMAX_METHOD: &str = "sX";
const NULL_METHOD: &str = "sN";
const SYNSCAN_OPEN_FLAG: isize = (TcpFlags::SYN as isize) | (TcpFlags::ACK as isize);
const SYNSCAN_CLOSE_FLAG: isize = (TcpFlags::RST as isize) | (TcpFlags::ACK as isize);
const FALLBACKSAN_CLOSE_FLAG: isize = TcpFlags::RST as isize;

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
    SynScan = TcpFlags::SYN as isize,
    FinScan = TcpFlags::FIN as isize,
    XmasScan = (TcpFlags::FIN | TcpFlags::URG | TcpFlags::PSH) as isize,
    NullScan = 0,
}

impl ScanType {
    fn scan_result(self: &ScanType, tcp_flags: isize) -> ScanResult {
        match self {
            ScanType::SynScan => ScanType::syn_scan(tcp_flags),
            _ => ScanType::scan_fallback(tcp_flags),
        }
    }

    fn syn_scan(tcp_flags: isize) -> ScanResult {
        match tcp_flags {
            SYNSCAN_OPEN_FLAG => ScanResult::Open,
            SYNSCAN_CLOSE_FLAG => ScanResult::Closed,
            _ => ScanResult::Unknown,
        }
    }

    fn scan_fallback(tcp_flags: isize) -> ScanResult {
        match tcp_flags {
            FALLBACKSAN_CLOSE_FLAG => ScanResult::Closed,
            _ => ScanResult::Unknown,
        }
    }
}

struct ScanedPacket {
    dest_port: u16,
    source_port: u16,
    scan_result: ScanResult,
}
#[derive(PartialEq, Debug)]
enum ScanResult {
    Open,
    Closed,
    Unknown,
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
                SYN_METHOD => ScanType::SynScan,
                FIN_METHOD => ScanType::FinScan,
                XMAX_METHOD => ScanType::XmasScan,
                NULL_METHOD => ScanType::NullScan,
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

    let (mut ts, _tr) = transport::transport_channel(
        1024,
        TransportChannelType::Layer4(TransportProtocol::Ipv4(IpNextHeaderProtocols::Tcp)),
    )
    .expect("Failed to open channel.");

    rayon::join(
        || send_packets(&mut ts, &packet_info),
        || receive_packets(&packet_info),
    );
}

fn send_packets(ts: &mut transport::TransportSender, packet_info: &PacketInfo) {
    let mut packet = build_packet(packet_info);
    for port in 1..packet_info.maximum_port {
        let mut tcp_header = tcp::MutableTcpPacket::new(&mut packet).unwrap();
        reregister_destination_port(port, &mut tcp_header, packet_info);
        thread::sleep(time::Duration::from_millis(10));
        let _size = ts
            .send_to(tcp_header, net::IpAddr::V4(packet_info.target_ipaddr))
            .expect("failed to send a packet");
        //println!("Port {} sent with size {}", port, size);
    }
}

fn receive_packets(packet_info: &PacketInfo) {
    let res = match packet_info.scan_type {
        ScanType::SynScan => receive_syn_packets(packet_info),
        ScanType::FinScan | ScanType::XmasScan | ScanType::NullScan => {
            receive_replied_packets(packet_info)
        }
    };

    for port in 1..packet_info.maximum_port + 1 {
        println!(
            "Port {}: {}",
            port,
            if res[port as usize] { "✅" } else { "❌" }
        );
    }
}

fn receive_syn_packets(packet_info: &PacketInfo) -> Vec<bool> {
    let mut scan_result = vec![false; (packet_info.maximum_port + 1) as usize];

    let mut rx = create_datalink_receiver();

    let mut count = 1;
    while count < packet_info.maximum_port {
        match rx.next() {
            Ok(frame) => {
                let packet = retrieve_tcp_packet_from_ethernet(
                    &EthernetPacket::new(frame).unwrap(),
                    packet_info,
                )
                .filter(|p| p.dest_port == packet_info.my_port);
                if let Some(p) = packet {
                    if p.source_port > packet_info.maximum_port {
                        panic!("Unexpected target port: {}", p.source_port);
                    }

                    scan_result[p.source_port as usize] = p.scan_result == ScanResult::Open;
                    //println!("Port {} is {:?}", p.source_port, p.scan_result);
                    count += 1;
                }
            }
            Err(e) => {
                eprintln!("Failed to read: {}", e);
            }
        }
    }

    scan_result
}

fn receive_replied_packets(packet_info: &PacketInfo) -> Vec<bool> {
    let mut scan_result = vec![true; (packet_info.maximum_port + 1) as usize];

    let mut rx = create_datalink_receiver();

    let mut count = 1;
    while count < packet_info.maximum_port {
        match rx.next() {
            Ok(frame) => {
                let packet = retrieve_tcp_packet_from_ethernet(
                    &EthernetPacket::new(frame).unwrap(),
                    packet_info,
                )
                .filter(|p| p.dest_port == packet_info.my_port);
                if let Some(p) = packet {
                    if p.source_port > packet_info.maximum_port {
                        panic!("Unexpected target port: {}", p.source_port);
                    }

                    scan_result[p.source_port as usize] = false;
                    println!("Port {} is {:?}", p.source_port, p.scan_result);
                    count += 1;
                }
            }
            Err(e) => {
                eprintln!("Failed to read: {}", e);
                count += 1;
            }
        }
    }

    scan_result
}

fn create_datalink_receiver() -> Box<dyn DataLinkReceiver> {
    let interface = datalink::interfaces()
        .into_iter()
        .find(|iface| iface.name == NIF_NAME)
        .expect("Failed to retrieve interface");

    let mut config = Config::default();
    config.read_timeout = Some(time::Duration::from_millis(10));
    let (_tx, rx) = match datalink::channel(&interface, config) {
        Ok(Ethernet(tx, rx)) => (tx, rx),
        _ => panic!("Failed to create datalink channel"),
    };

    rx
}

fn retrieve_tcp_packet_from_ethernet(
    p: &EthernetPacket,
    packet_info: &PacketInfo,
) -> Option<ScanedPacket> {
    match p.get_ethertype() {
        EtherTypes::Ipv4 => {
            retrieve_tcp_packet_from_ipv4(&Ipv4Packet::new(p.payload()).unwrap(), packet_info)
        }
        _ => None,
    }
}

fn retrieve_tcp_packet_from_ipv4(p: &Ipv4Packet, packet_info: &PacketInfo) -> Option<ScanedPacket> {
    match p.get_next_level_protocol() {
        IpNextHeaderProtocols::Tcp => {
            let p = p.payload();
            let p = TcpPacket::new(p).unwrap();
            let source_port = p.get_source() as u16;
            let dest_port = p.get_destination() as u16;
            let status = packet_info.scan_type.scan_result(p.get_flags() as isize);
            Some(ScanedPacket {
                source_port,
                dest_port,
                scan_result: status,
            })
        }
        _ => None,
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
