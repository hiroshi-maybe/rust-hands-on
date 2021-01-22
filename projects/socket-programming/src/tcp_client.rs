use std::io::{self, BufRead, BufReader, Write};
use std::net::TcpStream;
use std::str;

// Connect to specified IP address and port number with TCP connection
pub fn connect(address: &str) -> Result<(), failure::Error> {
    let mut stream = TcpStream::connect(address).expect("Couldn't connect to the server...");
    loop {
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        stream.write_all(input.as_bytes())?;

        let mut reader = BufReader::new(&stream);
        let mut buffer = Vec::new();
        reader.read_until(b'\n', &mut buffer)?;
        println!("Response: {}", str::from_utf8(&buffer)?);
    }
}
