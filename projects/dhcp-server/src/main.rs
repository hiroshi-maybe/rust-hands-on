use std::{
    env,
    net::{Ipv4Addr, UdpSocket},
    sync::Arc,
    thread,
};

use byteorder::{BigEndian, ByteOrder};
use dhcp::{DhcpPacket, DhcpServer};
use log::{debug, error, info};
use pnet::util::MacAddr;
mod database;
mod dhcp;
mod util;

const DHCP_SIZE: usize = 400;

const BOOTREQUEST: u8 = 1;
const BOOTREPLY: u8 = 2;

const HTYPE_ETHER: u8 = 1;
const MACADDR_SIZE: u8 = 6;

const DHCPDISCOVER: u8 = 1;
const DHCPOFFER: u8 = 2;
const DHCPREQUEST: u8 = 3;
const DHCPACK: u8 = 5;
const DHCPNAK: u8 = 6;
const DHCPRELEASE: u8 = 7;

enum Code {
    MessageType = 53,
    IPAddressLeaseTime = 51,
    ServerIdentifier = 54,
    RequestedIpAddress = 50,
    SubnetMask = 1,
    Router = 3,
    DNS = 6,
    End = 255,
}

fn main() {
    env::set_var("RUST_LOG", "debug");
    env_logger::init();

    let server_socket = UdpSocket::bind("0.0.0.0:67").expect("Failed to bind socket");
    server_socket.set_broadcast(true).unwrap();

    let dhcp_server = Arc::new(
        DhcpServer::new().unwrap_or_else(|e| panic!("Failed to start dhcp server. {:?}", e)),
    );

    loop {
        let mut recv_buf = [0u8; 1024];
        match server_socket.recv_from(&mut recv_buf) {
            Ok((size, src)) => {
                debug!("received data from {}, size: {}", src, size);
                let transmission_socket = server_socket
                    .try_clone()
                    .expect("Failed to create client socket");
                let cloned_dhcp_server = dhcp_server.clone();

                thread::spawn(move || {
                    if let Some(dhcp_packet) = DhcpPacket::new(recv_buf[..size].to_vec()) {
                        if dhcp_packet.get_op() != BOOTREQUEST {
                            return;
                        }

                        if let Err(e) =
                            dhcp_handler(&dhcp_packet, &&transmission_socket, cloned_dhcp_server)
                        {
                            error!("{}", e);
                        }
                    }
                });
            }
            Err(e) => {
                error!("Could not receive a datagram: {}", e);
            }
        }
    }
}

fn dhcp_handler(
    packet: &DhcpPacket,
    soc: &UdpSocket,
    dhcp_server: Arc<DhcpServer>,
) -> Result<(), failure::Error> {
    let message = packet
        .get_option(Code::MessageType as u8)
        .ok_or_else(|| failure::err_msg("specified option was not found"))?;
    let message_type = message[0];
    let tx_id = BigEndian::read_u32(packet.get_xid());
    let client_macaddr = packet.get_chaddr();
    match message_type {
        DHCPDISCOVER => dhcp_discover_message_handler(tx_id, dhcp_server, packet, soc),
        DHCPREQUEST => match packet.get_option(Code::ServerIdentifier as u8) {
            Some(server_id) => dhcp_request_message_handler_responded_to_offer(
                tx_id,
                dhcp_server,
                packet,
                client_macaddr,
                soc,
                server_id,
            ),
            None => dhcp_request_message_handler_to_reallocate(
                tx_id,
                dhcp_server,
                packet,
                client_macaddr,
                soc,
            ),
        },
        DHCPRELEASE => dhcp_release_message_handler(tx_id, dhcp_server, packet, client_macaddr),
        _ => {
            let msg = format!(
                "{:x}: received unimplemented message, message_type: {}",
                tx_id, message_type
            );
            Err(failure::err_msg(msg))
        }
    }
}

fn dhcp_discover_message_handler(
    tx_id: u32,
    dhcp_server: Arc<DhcpServer>,
    received_packet: &DhcpPacket,
    soc: &UdpSocket,
) -> Result<(), failure::Error> {
    info!("{:x}: received DHCPDISCOVER", tx_id);

    let ip_to_be_leased = select_lease_ip(&dhcp_server, &received_packet)?;
    Ok(())
}

fn select_lease_ip(
    dhcp_server: &Arc<DhcpServer>,
    received_packet: &DhcpPacket,
) -> Result<Ipv4Addr, failure::Error> {
    let con = dhcp_server.db_connection.lock().unwrap();
    if let Some(ip_from_used) = database::select_entry(&con, received_packet.get_chaddr())? {
        if dhcp_server.network_addr.contains(ip_from_used) && util::is_ipaddr_available(ip_from_used).is_ok() {
            return Ok(ip_from_used);
        }
    }

    if let Some(ip_to_be_leased) = obtain_available_ip_from_requested_option(dhcp_server, &received_packet) {
        return Ok(ip_to_be_leased);
    }

    while let Some(ip_addr) = dhcp_server.pick_available_ip() {
        if util::is_ipaddr_available(ip_addr).is_ok() {
            return Ok(ip_addr);
        }
    }
    Err(failure::err_msg("Could not obtain available ip address."))
}

fn obtain_available_ip_from_requested_option(
    dhcp_server: &Arc<DhcpServer>,
    received_packet: &DhcpPacket,
) -> Option<Ipv4Addr> {
    let ip = received_packet.get_option(Code::RequestedIpAddress as u8)?;
    let requested_ip = util::u8_to_ipv4addr(&ip)?;
    let ip_from_pool = dhcp_server.pick_specified_ip(requested_ip)?;
    if util::is_ipaddr_available(ip_from_pool).is_ok() {
        Some(requested_ip)
    } else {
        None
    }
}

fn dhcp_request_message_handler_responded_to_offer(
    tx_id: u32,
    dhcp_server: Arc<DhcpServer>,
    packet: &DhcpPacket,
    client_macaddr: MacAddr,
    soc: &UdpSocket,
    server_id: Vec<u8>,
) -> Result<(), failure::Error> {
    Ok(())
}

fn dhcp_request_message_handler_to_reallocate(
    tx_id: u32,
    dhcp_server: Arc<DhcpServer>,
    packet: &DhcpPacket,
    client_macaddr: MacAddr,
    soc: &UdpSocket,
) -> Result<(), failure::Error> {
    Ok(())
}

fn dhcp_release_message_handler(
    tx_id: u32,
    dhcp_server: Arc<DhcpServer>,
    packet: &DhcpPacket,
    client_macaddr: MacAddr,
) -> Result<(), failure::Error> {
    Ok(())
}

fn make_dhcp_packet(
    received_packet: &DhcpPacket,
    dhcp_server: &Arc<DhcpServer>,
    message_type: u8,
    ip_to_be_leased: Ipv4Addr,
) -> Result<DhcpPacket, failure::Error> {
    let buffer = vec![0u8; DHCP_SIZE];
    let mut dhcp_packet = DhcpPacket::new(buffer).unwrap();

    dhcp_packet.set_op(BOOTREPLY);
    dhcp_packet.set_htype(HTYPE_ETHER);
    dhcp_packet.set_hlen(6);
    dhcp_packet.set_xid(received_packet.get_xid());
    if message_type == DHCPACK {
        dhcp_packet.set_ciaddr(received_packet.get_ciaddr());
    }
    dhcp_packet.set_yiaddr(ip_to_be_leased);
    dhcp_packet.set_flags(received_packet.get_flags());
    dhcp_packet.set_giaddr(received_packet.get_giaddr());
    dhcp_packet.set_chaddr(received_packet.get_chaddr());

    let mut cursor = dhcp::OPTIONS;
    dhcp_packet.set_magic_cookie(&mut cursor);
    dhcp_packet.set_option(
        &mut cursor,
        Code::MessageType as u8,
        1,
        Some(&[message_type]),
    );
    dhcp_packet.set_option(
        &mut cursor,
        Code::IPAddressLeaseTime as u8,
        4,
        Some(&dhcp_server.lease_time),
    );
    dhcp_packet.set_option(
        &mut cursor,
        Code::ServerIdentifier as u8,
        4,
        Some(&dhcp_server.server_address.octets()),
    );
    dhcp_packet.set_option(
        &mut &mut cursor,
        Code::SubnetMask as u8,
        4,
        Some(&dhcp_server.subnet_mask.octets()),
    );
    dhcp_packet.set_option(
        &mut &mut cursor,
        Code::Router as u8,
        4,
        Some(&dhcp_server.default_gateway.octets()),
    );
    dhcp_packet.set_option(
        &mut &mut cursor,
        Code::DNS as u8,
        4,
        Some(&dhcp_server.dns_server.octets()),
    );
    dhcp_packet.set_option(&mut cursor, Code::End as u8, 0, None);

    Ok(dhcp_packet)
}
