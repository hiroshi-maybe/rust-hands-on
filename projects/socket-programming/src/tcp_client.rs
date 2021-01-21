use std::io::{self, Read, Write};
use std::net::TcpStream;
use std::str;

// Connect to specified IP address and port number with TCP connection
pub fn connect(address: &str) -> Result<(), failure::Error> {
    let mut stream = TcpStream::connect(address).expect("Couldn't connect to the server...");
    loop {
        let mut wbuffer = [0u8; 1024];
        let stdin = io::stdin();
        let mut handle = stdin.lock();
        let bytes = handle.read(&mut wbuffer)?;

        if bytes == 0 {
            return Ok(());
        }

        stream.write_all(&wbuffer[..bytes])?;

        let mut rbuffer = [0u8; 1024];
        let bytes = stream.read(&mut rbuffer)?;
        print!("Response: {}", str::from_utf8(&rbuffer[..bytes])?);
    }
}
