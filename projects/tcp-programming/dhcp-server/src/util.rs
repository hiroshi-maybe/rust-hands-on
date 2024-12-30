use std::{collections::HashMap, fs, io, net::{AddrParseError, IpAddr, Ipv4Addr}, str::FromStr, sync::mpsc, thread, time::Duration};

use byteorder::{BigEndian, WriteBytesExt};
use log::{debug, info};
use pnet::{packet::{Packet, icmp::{IcmpTypes, echo_request::{EchoRequestPacket, MutableEchoRequestPacket}}, ip::IpNextHeaderProtocols}, transport::{self, TransportChannelType, icmp_packet_iter, TransportProtocol::Ipv4}, util::checksum};

pub const NETWORK_ADDR_KEY: &str = "NETWORK_ADDR";
pub const SUBNET_MASK_KEY: &str = "SUBNET_MASK";
pub const SERVER_IDENTIFIER_KEY: &str = "SERVER_IDENTIFIER";
pub const DEFAULT_GATEWAY_KEY: &str = "DEFAULT_GATEWAY";
pub const DNS_SERVER_KEY: &str = "DNS_SERVER";
pub const LEASE_TIME_KEY: &str = "LEASE_TIME";

pub fn load_env() -> HashMap<String, String> {
    return fs::read_to_string(".env")
        .expect("Failed to read .env file")
        .lines()
        .filter_map(|line| {
            let tokens = line.split("=").map(str::trim).collect::<Vec<_>>();
            match tokens.len() {
                2 => Some((tokens[0].to_string(), tokens[1].to_string())),
                _ => None,
            }
        })
        .collect();
}

pub fn obtain_static_addresses(
    env: &HashMap<String, String>,
) -> Result<HashMap<String, Ipv4Addr>, AddrParseError> {
    let mut map = HashMap::new();

    parse_and_insert(NETWORK_ADDR_KEY, env, &mut map)?;
    parse_and_insert(SUBNET_MASK_KEY, env, &mut map)?;
    parse_and_insert(SERVER_IDENTIFIER_KEY, env, &mut map)?;
    parse_and_insert(DEFAULT_GATEWAY_KEY, env, &mut map)?;
    parse_and_insert(DNS_SERVER_KEY, env, &mut map)?;

    Ok(map)
}

fn parse_and_insert(
    key: &str,
    env: &HashMap<String, String>,
    store: &mut HashMap<String, Ipv4Addr>,
) -> Result<(), AddrParseError> {
    let addr = get_and_parse_addr(key, env)?;
    store.insert(key.to_string(), addr);

    Ok(())
}

pub fn get_and_parse_addr<F>(
    key: &str,
    env: &HashMap<String, String>,
) -> Result<F, <F as FromStr>::Err>
where
    F: FromStr,
{
    env.get(key)
        .expect(format!("Missing {:?} entry", key).as_str())
        .parse::<F>()
}

pub fn make_big_endian_vec_from_u32(i: u32) -> Result<Vec<u8>, io::Error> {
    let mut v = Vec::new();
    v.write_u32::<BigEndian>(i)?;
    Ok(v)
}

pub fn is_ipaddr_available(target_ip: Ipv4Addr) -> Result<(), failure::Error> {
    let icmp_buf = create_default_icmp_buffer();
    let icmp_packet = EchoRequestPacket::new(&icmp_buf).unwrap();

    let (mut transport_sender, mut transport_receiver) = transport::transport_channel(1024, TransportChannelType::Layer4(Ipv4(IpNextHeaderProtocols::Icmp)),
    )?;
    transport_sender.send_to(icmp_packet, IpAddr::V4(target_ip))?;

    let (sender, receiver) = mpsc::channel();

    thread::spawn(move || {
        let mut iter = icmp_packet_iter(&mut transport_receiver);
        let (packet, _) = iter.next().unwrap();
        if packet.get_icmp_type() == IcmpTypes::EchoReply {
            match sender.send(true) {
                Err(_) => {
                    info!("icmp timeout");
                }
                _ => {
                    return;
                }
            }
        }
    });

    if receiver.recv_timeout(Duration::from_millis(200)).is_ok() {
        Err(failure::format_err!("Ip addr already in use: {}", target_ip))
    } else {
        debug!("not received reply within timeout");
        Ok(())
    }
}

fn create_default_icmp_buffer() -> [u8; 8] {
    let mut buffer = [0u8; 8];
    let mut icmp_packet = MutableEchoRequestPacket::new(&mut buffer).unwrap();
    icmp_packet.set_icmp_type(IcmpTypes::EchoRequest);
    let checksum = checksum(icmp_packet.to_immutable().packet(), 16);
    icmp_packet.set_checksum(checksum);
    buffer
}

pub fn u8_to_ipv4addr(buf: &[u8]) -> Option<Ipv4Addr> {
    if buf.len() == 4 {
        Some(Ipv4Addr::new(buf[0], buf[1], buf[2], buf[3]))
    } else {
        None
    }
}