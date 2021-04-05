use std::{net::Ipv4Addr, ops::Range};

use pnet::{packet::PrimitiveValues, util::MacAddr};

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
}

fn find_field_range(field: usize) -> Range<usize> {
    let p = PACKET_FORMAT
        .iter()
        .position(|&f| f == field)
        .expect("Field position not found");
    field..PACKET_FORMAT[p + 1]
}

pub struct DhcpServer {
    pub server_address: Ipv4Addr,
    pub default_gateway: Ipv4Addr,
    pub subnet_mask: Ipv4Addr,
    pub dns_server: Ipv4Addr,
    pub lease_time: Vec<u8>,
}

impl DhcpServer {
    pub fn new() -> Result<DhcpServer, failure::Error> {
        let dummyAddr = Ipv4Addr::new(127, 0, 0, 1);
        Ok(DhcpServer {
            server_address: dummyAddr,
            default_gateway: dummyAddr,
            subnet_mask: dummyAddr,
            dns_server: dummyAddr,
            lease_time: Vec::new(),
        })
    }
}
