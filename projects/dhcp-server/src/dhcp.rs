const OP: usize = 0;

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
}

pub struct DhcpServer {}

impl DhcpServer {
    pub fn new() -> Result<DhcpServer, failure::Error> {
        Ok(DhcpServer {})
    }
}
