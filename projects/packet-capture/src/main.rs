use log::{error, info};
use pnet::datalink;
use pnet::datalink::Channel::Ethernet;
use pnet::packet::ethernet::{EthernetPacket, EtherTypes};
use pnet::packet::ip::IpNextHeaderProtocols;
use pnet::packet::ipv4::Ipv4Packet;
use pnet::packet::ipv6::Ipv6Packet;
use pnet::packet::tcp::TcpPacket;
use pnet::packet::udp::UdpPacket;
use pnet::packet::Packet;
use std::env;

mod packets;
use packets::GettableEndPoints;

const WIDTH: usize = 20;

/// # How to run for Mac OS?
///
/// Install command line tool
/// $ brew install iproute2mac
/// Show a list of network interfaces. Look at MAC address in the Network settings and find corresponding network interface.
/// $ ip link show
///
///
/// $ cargo build
/// $ sudo ./target/debug/packet-capture en0

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

    let (_tx, mut rx) = match datalink::channel(&interface, Default::default()) {
        Ok(Ethernet(tx, rx)) => (tx, rx),
        Ok(_) => panic!("Unhandled channel type"),
        Err(e) => panic!("Failed to create datalink channel {}", e),
    };

    loop {
        match rx.next() {
            Ok(frame) => {
                // Build ethernet packet
                let frame = EthernetPacket::new(frame).unwrap();
                match frame.get_ethertype() {
                    EtherTypes::Ipv4 => {
                        ipv4_handler(&frame);
                    },
                    EtherTypes::Ipv6 => {
                        ipv6_handler(&frame);
                    },
                    _ => {
                        info!("Neither IPv4 nor IPv6 packet");
                    }
                }
            },
            Err(e) => {
                error!("Failed to read: {}", e);
            }
        }
    }
}

// Build IPv4 packet from ethernet packet
fn ipv4_handler(ethernet: &EthernetPacket) {
    match Ipv4Packet::new(ethernet.payload()) {
        Some(packet) => {
            match packet.get_next_level_protocol() {
                IpNextHeaderProtocols::Tcp => {
                    tcp_handler(&packet);
                },
                IpNextHeaderProtocols::Udp => {
                    udp_handler(&packet);
                },
                _ => {
                    info!("Neither TCP nor UDP packet")
                }
            }
        },
        None => {
            error!("Failed to restore IPv4 packet");
        }
    }
}
fn ipv6_handler(ethernet: &EthernetPacket) {
    match Ipv6Packet::new(ethernet.payload()) {
        Some(packet) => {
            match packet.get_next_header() {
                IpNextHeaderProtocols::Tcp => {
                    tcp_handler(&packet);
                },
                IpNextHeaderProtocols::Udp => {
                    udp_handler(&packet);
                },
                _ => {
                    info!("Neither TCP nor UDP packet")
                }
            }
        },
        None => {
            error!("Failed to restore IPv4 packet");
        }
    }

}
fn tcp_handler(packet: &dyn GettableEndPoints) {
    let tcp = TcpPacket::new(packet.get_payload());
    if let Some(tcp) = tcp {
        print_packet_info(packet, &tcp, "TCP");
    }
}

fn udp_handler(packet: &dyn GettableEndPoints) {
    let udp = UdpPacket::new(packet.get_payload());
    if let Some(udp) = udp {
        print_packet_info(packet, &udp, "UDP");
    }
}

fn print_packet_info(l3: &dyn GettableEndPoints, l4: &dyn GettableEndPoints, proto: &str) {
    println!(
        "Captured a {} packet from {}|{} to {}|{}\n",
        proto,
        l3.get_source(),
        l4.get_source(),
        l3.get_destination(),
        l4.get_destination()
    );
    let payload = l4.get_payload();
    let len = payload.len();

    for i in 0..len {
        print!("{:<02X} ", payload[i]);
        if i % WIDTH == WIDTH - 1 || i == len - 1 {
            for _j in 0..WIDTH - 1 - (i % (WIDTH)) {
                print!("     ");
            }
            print!("|   ");
            for j in i - i % WIDTH..=i {
                if payload[j].is_ascii_alphabetic() {
                    print!("{}", payload[j] as char);
                } else {
                    print!(".");
                }
            }
            println!();
        }
    }
    println!("{}","=".repeat(WIDTH * 3));
    println!();
}