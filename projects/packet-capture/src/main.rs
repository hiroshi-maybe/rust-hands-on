use log::{error, info};
use pnet::datalink;
use pnet::datalink::Channel::Ethernet;
use pnet::packet::ethernet::{EthernetPacket, EtherTypes};
use pnet::packet::ip::IpNextHeaderProtocols;
use pnet::packet::ipv4::Ipv4Packet;
use pnet::packet::Packet;
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
                    //tcp_handler(&packet);
                },
                IpNextHeaderProtocols::Udp => {
                    //udp_handler(&packet);
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
fn ipv6_handler(ethernet: &EthernetPacket) {}
//fn tcp_handler(packet: &GettableEndPoints) {}
