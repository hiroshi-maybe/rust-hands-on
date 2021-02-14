use pnet::datalink;
use pnet::datalink::Channel::Ethernet;
use pnet::packet::ethernet::{EtherTypes, EthernetPacket};
use pnet::packet::ip::IpNextHeaderProtocols;
use pnet::packet::ipv4::Ipv4Packet;
use pnet::packet::tcp::{self, MutableTcpPacket, TcpFlags, TcpPacket};
use pnet::packet::Packet;
use pnet::transport::{self, TransportChannelType, TransportProtocol};
use std::time::Duration;
use std::net;
use std::thread;

/// Usage:
/// $ cargo build
/// $ sudo ./target/debug/tcp-syn-sock

const NIF_NAME: &str = "en0";
const TARGET_IPADDR: &str = "10.0.1.1";
const TCP_SIZE: usize = 20;
const SRC_PORT: u16 = 33333;
const DEST_PORT: u16 = 53;
const OPEN_FLAG: isize = (TcpFlags::SYN as isize) | (TcpFlags::ACK as isize);
const CLOSE_FLAG: isize = (TcpFlags::RST as isize) | (TcpFlags::ACK as isize);

/// I figured out that raw IP datagram is never passed in Mac OS (Free BSD).
/// https://stackoverflow.com/questions/6878603/strange-raw-socket-on-mac-os-x
/// https://sock-raw.org/papers/sock_raw

fn main() {
    let h = thread::spawn(move || {
        rcv_packets_with_bpf();
    });

    thread::sleep(Duration::from_secs(1));
    send_raw_ip_datagram();

    h.join().expect("The thread being joined has panicked");
}

fn rcv_packets_with_bpf() {
    let interface = datalink::interfaces().into_iter().find(|iface| iface.name == NIF_NAME)
    .expect("Failed to retrieve interface");
    let (_tx, mut rx) = match datalink::channel(&interface, Default::default()) {
        Ok(Ethernet(tx, rx)) => (tx, rx),
        _ => panic!("Failed to create datalink channel")
    };

    loop {
        match rx.next() {
            Ok(frame) => {
                retrieve_tcp_packet_from_ethernet(&EthernetPacket::new(frame).unwrap());
            },
            Err(e) => {
                eprintln!("Failed to read: {}", e);
            }
        }
    }
}

fn retrieve_tcp_packet_from_ethernet(p: &EthernetPacket) {
    match p.get_ethertype() {
        EtherTypes::Ipv4 => {
            retrieve_tcp_packet_from_ipv4(&Ipv4Packet::new(p.payload()).unwrap());
        },
        _ => {}
    }
}

fn retrieve_tcp_packet_from_ipv4(p: &Ipv4Packet) {
    match p.get_next_level_protocol() {
        IpNextHeaderProtocols::Tcp => {
            let p = p.payload();
            let p = TcpPacket::new(p).unwrap();
            if p.get_destination().to_string() == SRC_PORT.to_string() {
                let status = match p.get_flags() as isize {
                    OPEN_FLAG => "open",
                    CLOSE_FLAG=> "closed",
                    _ => "unknown"
                };
                println!("port {} is {}", p.get_source().to_string(), status);
            }
        },
        _ => {}
    }
}

fn send_raw_ip_datagram() {
    // https://stackoverflow.com/questions/6878603/strange-raw-socket-on-mac-os-x
    // https://docs.rs/pnet_transport/0.22.0/src/pnet_transport/lib.rs.html#106
    //
    // pnet_sys::socket(AF_INET, SOCK_RAW, 6)
    //  -> libc::socket(libc::AF_INET = 2, libc::SOCK_RAW = 3, 6)
    //
    // - AF_INET = 2 from https://github.com/rust-lang/libc/blob/2a2196d6dc2aec3ef0b0bcd11231db1f3b2ca04e/src/unix/bsd/apple/mod.rs#L2173
    // - SOCK_RAW = 3 from https://github.com/rust-lang/libc/blob/2a2196d6dc2aec3ef0b0bcd11231db1f3b2ca04e/src/unix/bsd/apple/mod.rs#L2258
    // `6` is a protocol number for TCP http://www.iana.org/assignments/protocol-numbers/protocol-numbers.xhtml
    let (mut ts, _tr) = transport::transport_channel(
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
}
