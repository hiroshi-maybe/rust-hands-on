use std::net::UdpSocket;
use std::{io, str};

pub fn communicate(address: &str) -> Result<(), failure::Error> {
    let socket = UdpSocket::bind("127.0.0.1:0")?;
    loop {
        let mut input = String::new();
        println!("Waiting for next input...");
        io::stdin().read_line(&mut input)?;
        // Default maximum size to be sent through UDP in Mac OS.
        // https://stackoverflow.com/questions/22819214/udp-message-too-long
        // let input = "a".to_string().repeat(9216);

        socket.send_to(input.as_bytes(), address)?;
        let mut buffer = [0u8; 1024];
        let (size, src) = socket.recv_from(&mut buffer).expect("Failed to receive");
        debug!("Received data from {} with size {}", src, size);
        println!(
            "{}",
            str::from_utf8(&buffer).expect("Failed to convert to String")
        );
    }
}
