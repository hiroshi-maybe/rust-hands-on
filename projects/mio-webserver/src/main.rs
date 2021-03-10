use mio::tcp::{TcpListener, TcpStream};
use mio::{Events, Poll, PollOpt, Ready, Token};
use std::collections::HashMap;

const SERVER: Token = Token(0);

struct WebServer {
    listening_socket: TcpListener,
    connections: HashMap<usize, TcpStream>,
    next_connection_id: usize,
}

impl WebServer {
    fn new(addr: &str) -> Result<Self, failure::Error> {
        let address = addr.parse()?;
        let listening_socket = TcpListener::bind(&address)?;
        Ok(WebServer {
            listening_socket,
            connections: HashMap::new(),
            next_connection_id: 1,
        })
    }

    fn run(&mut self) -> Result<(), failure::Error> {
        let poll = Poll::new()?;

        poll.register(
            &self.listening_socket,
            SERVER,
            Ready::readable(),
            PollOpt::level(),
        )?;

        let mut events = Events::with_capacity(1024);
        let mut response = Vec::new();

        Ok(())
    }
}

fn main() {
    println!("Hello, world!");
}
