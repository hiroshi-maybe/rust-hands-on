use pnet::packet::ip::IpNextHeaderProtocols;
use pnet::packet::tcp::{self, MutableTcpPacket, TcpFlags, TcpPacket};
use pnet::transport::{self, TransportChannelType, TransportProtocol};
use std::net::{self, Ipv4Addr};
use std::thread;

const TARGET_IPADDR: &str = "10.0.1.1";
const TCP_SIZE: usize = 20;
const SRC_PORT: u16 = 33333;
const DEST_PORT: u16 = 47;

fn main() {
    let (mut ts, mut tr) = transport::transport_channel(
        1024,
        TransportChannelType::Layer4(TransportProtocol::Ipv4(IpNextHeaderProtocols::Tcp)),
    )
    .expect("Failed to open channel.");

    let mut tcp_buffer = [0u8; TCP_SIZE];
    let mut tcp_header = MutableTcpPacket::new(&mut tcp_buffer[..]).unwrap();
    tcp_header.set_source(SRC_PORT);
    tcp_header.set_data_offset(5);
    tcp_header.set_flags(TcpFlags::SYN);
    tcp_header.set_destination(DEST_PORT);
    let target_ipaddr = TARGET_IPADDR.parse().expect("invalid ipaddr");
    let checksum = tcp::ipv4_checksum(
        &tcp_header.to_immutable(),
        &"10.0.1.2".parse().expect("invalid ipaddr"),
        &target_ipaddr
    );
    tcp_header.set_checksum(checksum);
    let size = ts.send_to(tcp_header, net::IpAddr::V4(target_ipaddr))
        .expect("failed to send a packet");
    println!("Port {} sent with size {}", DEST_PORT, size);

    let mut packet_iter = transport::tcp_packet_iter(&mut tr);
    loop {
        let (tcp_packet, _) = packet_iter.next().expect("Error to receive a packet");
        if tcp_packet.get_destination() == SRC_PORT {
            println!("Received a packet");
        }
    }
}
