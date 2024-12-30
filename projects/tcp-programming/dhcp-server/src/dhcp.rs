use std::{
    collections::HashMap,
    net::Ipv4Addr,
    ops::Range,
    sync::{Mutex, RwLock},
};

use ipnetwork::Ipv4Network;
use log::info;
use pnet::{packet::PrimitiveValues, util::MacAddr};
use rusqlite::Connection;

use super::database;
use super::util;

const OP: usize = 0;
const HTYPE: usize = 1;
const HLEN: usize = 2;
const HOPS: usize = 3;
const XID: usize = 4;
const SECS: usize = 8;
const FLAGS: usize = 10;
const CIADDR: usize = 12;
const YIADDR: usize = 16;
const SIADDR: usize = 20;
const GIADDR: usize = 24;
const CHADDR: usize = 28;
const SNAME: usize = 44;
const FILE: usize = 108;
pub const OPTIONS: usize = 236;

const OPTION_END: u8 = 255;

const PACKET_FORMAT: [usize; 15] = [
    OP, HTYPE, HLEN, HOPS, XID, SECS, FLAGS, CIADDR, YIADDR, SIADDR, GIADDR, CHADDR, SNAME, FILE,
    OPTIONS,
];

pub struct DhcpPacket {
    buffer: Vec<u8>,
}
impl DhcpPacket {
    pub fn new(buffer: Vec<u8>) -> Option<DhcpPacket> {
        Some(DhcpPacket { buffer })
    }

    pub fn get_op(&self) -> u8 {
        self.buffer[OP]
    }

    pub fn set_op(&mut self, op: u8) {
        self.buffer[OP] = op;
    }

    pub fn set_htype(&mut self, hlen: u8) {
        self.buffer[HTYPE] = hlen;
    }

    pub fn set_hlen(&mut self, hlen: u8) {
        self.buffer[HLEN] = hlen;
    }

    pub fn set_xid(&mut self, xid: &[u8]) {
        self.buffer[find_field_range(XID)].copy_from_slice(xid);
    }

    pub fn set_flags(&mut self, flags: &[u8]) {
        self.buffer[find_field_range(FLAGS)].copy_from_slice(flags);
    }

    pub fn set_ciaddr(&mut self, ciaddr: Ipv4Addr) {
        self.buffer[find_field_range(CIADDR)].copy_from_slice(&ciaddr.octets());
    }

    pub fn set_yiaddr(&mut self, yiaddr: Ipv4Addr) {
        self.buffer[find_field_range(YIADDR)].copy_from_slice(&yiaddr.octets());
    }

    pub fn set_giaddr(&mut self, giaddr: Ipv4Addr) {
        self.buffer[find_field_range(GIADDR)].copy_from_slice(&giaddr.octets())
    }

    pub fn set_chaddr(&mut self, chaddr: MacAddr) {
        let t = chaddr.to_primitive_values();
        let macaddr_value = [t.0, t.1, t.2, t.3, t.4, t.5];
        self.buffer[CHADDR..CHADDR + macaddr_value.len()].copy_from_slice(&macaddr_value);
    }

    pub fn set_magic_cookie(&mut self, cursor: &mut usize) {
        self.buffer[*cursor..*cursor + 4].copy_from_slice(&[0x63, 0x82, 0x53, 0x63]);
        *cursor += 4;
    }

    pub fn set_option(
        &mut self,
        cursor: &mut usize,
        code: u8,
        len: usize,
        contents: Option<&[u8]>,
    ) {
        self.buffer[*cursor] = code;
        if code == OPTION_END {
            return;
        }

        *cursor += 1;
        self.buffer[*cursor] = len as u8;
        *cursor += 1;
        if let Some(contents) = contents {
            self.buffer[*cursor..*cursor + contents.len()].copy_from_slice(contents);
        }
        *cursor += len;
    }

    pub fn get_xid(&self) -> &[u8] {
        &self.buffer[find_field_range(XID)]
    }

    pub fn get_flags(&self) -> &[u8] {
        &self.buffer[find_field_range(FLAGS)]
    }

    pub fn get_ciaddr(&self) -> Ipv4Addr {
        let b = &self.buffer[find_field_range(CIADDR)];
        Ipv4Addr::new(b[0], b[1], b[2], b[3])
    }

    pub fn get_giaddr(&self) -> Ipv4Addr {
        let b = &self.buffer[find_field_range(GIADDR)];
        Ipv4Addr::new(b[0], b[1], b[2], b[3])
    }

    pub fn get_chaddr(&self) -> MacAddr {
        let b = &self.buffer[find_field_range(CHADDR)];
        MacAddr::new(b[0], b[1], b[2], b[3], b[4], b[5])
    }

    pub fn get_option(&self, option_code: u8) -> Option<Vec<u8>> {
        let mut i: usize = 4; /* cookie size */
        let options = self.get_options();
        while options[i] != OPTION_END {
            if options[i] == option_code {
                let len = options[i + 1];
                let buf_index = i + 2;
                return Some(options[buf_index..buf_index + len as usize].to_vec());
            } else if options[i] == 0 {
                // padding
                i += 1;
            } else {
                i += 1;
                let len = options[i];
                i += 1;
                i += len as usize;
            }
        }
        None
    }

    fn get_options(&self) -> &[u8] {
        &self.buffer[OPTIONS..]
    }
}

fn find_field_range(field: usize) -> Range<usize> {
    let p = PACKET_FORMAT
        .iter()
        .position(|&f| f == field)
        .expect("Field position not found");
    field..PACKET_FORMAT[p + 1]
}

pub struct DhcpServer {
    address_pool: RwLock<Vec<Ipv4Addr>>,
    pub db_connection: Mutex<Connection>,
    pub network_addr: Ipv4Network,
    pub server_address: Ipv4Addr,
    pub default_gateway: Ipv4Addr,
    pub subnet_mask: Ipv4Addr,
    pub dns_server: Ipv4Addr,
    pub lease_time: Vec<u8>,
}

impl DhcpServer {
    pub fn new() -> Result<DhcpServer, failure::Error> {
        let env = util::load_env();
        let static_addresses = util::obtain_static_addresses(&env)?;

        let subnet_mask = static_addresses[util::SUBNET_MASK_KEY];
        let network_addr_with_prefix: Ipv4Network = Ipv4Network::new(
            static_addresses[util::NETWORK_ADDR_KEY],
            ipnetwork::ipv4_mask_to_prefix(subnet_mask)?,
        )?;

        let con = Connection::open("dhcp.db")?;
        let addr_pool = Self::init_address_pool(&con, &static_addresses, network_addr_with_prefix)?;

        info!("Ther are {} addresses in the address pool", addr_pool.len());
        let raw_lease_time: u32 = util::get_and_parse_addr(util::LEASE_TIME_KEY, &env)?;
        let lease_time = util::make_big_endian_vec_from_u32(raw_lease_time)?;

        let dummyAddr = Ipv4Addr::new(127, 0, 0, 1);
        Ok(DhcpServer {
            address_pool: RwLock::new(addr_pool),
            db_connection: Mutex::new(con),
            network_addr: network_addr_with_prefix,
            server_address: static_addresses[util::SERVER_IDENTIFIER_KEY],
            default_gateway: static_addresses[util::DEFAULT_GATEWAY_KEY],
            subnet_mask: static_addresses[util::SUBNET_MASK_KEY],
            dns_server: static_addresses[util::DNS_SERVER_KEY],
            lease_time,
        })
    }

    pub fn pick_available_ip(&self) -> Option<Ipv4Addr> {
        let mut lock = self.address_pool.write().unwrap();
        lock.pop()
    }

    pub fn pick_specified_ip(&self, requesetd_ip: Ipv4Addr) -> Option<Ipv4Addr> {
        let mut lock = self.address_pool.write().unwrap();
        lock.iter()
            .position(|&a| a == requesetd_ip)
            .map(|i| lock.remove(i))
    }

    pub fn release_address(&self, release_ip: Ipv4Addr) {
        let mut lock = self.address_pool.write().unwrap();
        lock.insert(0, release_ip);
    }

    fn init_address_pool(
        con: &Connection,
        static_addresses: &HashMap<String, Ipv4Addr>,
        network_addr_with_prefix: Ipv4Network,
    ) -> Result<Vec<Ipv4Addr>, failure::Error> {
        let network_addr = static_addresses.get(util::NETWORK_ADDR_KEY).unwrap();
        let default_gateway = static_addresses.get(util::DEFAULT_GATEWAY_KEY).unwrap();
        let dhcp_server_addr = static_addresses.get(util::SERVER_IDENTIFIER_KEY).unwrap();
        let dns_server_addr = static_addresses.get(util::DNS_SERVER_KEY).unwrap();
        let broadcast = network_addr_with_prefix.broadcast();

        let mut used_ip_addrs = database::select_addresses(con, Some(0))?;
        used_ip_addrs.extend(vec![
            *network_addr,
            *default_gateway,
            *dhcp_server_addr,
            *dns_server_addr,
            broadcast,
        ]);
        let mut addr_pool: Vec<Ipv4Addr> = network_addr_with_prefix
            .iter()
            .filter(|addr| !used_ip_addrs.contains(addr))
            .collect::<Vec<_>>();
        addr_pool.reverse();

        Ok(addr_pool)
    }
}
