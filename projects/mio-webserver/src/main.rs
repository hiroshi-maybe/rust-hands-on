use mio::tcp::{TcpListener, TcpStream};
use mio::{Event, Events, Poll, PollOpt, Ready, Token};
use regex::Regex;
use std::collections::HashMap;
use std::{env, process, str};
use std::fs::File;
use std::io::{BufReader, Read, Write};
#[macro_use]
extern crate log;

const WEBROOT: &str = "/webroot";
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

        loop {
            match poll.poll(&mut events, None) {
                Ok(_) => {}
                Err(e) => {
                    error!("{}", e);
                    continue;
                }
            }

            for event in &events {
                match event.token() {
                    SERVER => {
                        let (stream, remote) = match self.listening_socket.accept() {
                            Ok(t) => t,
                            Err(e) => {
                                error!("{}", e);
                                continue;
                            }
                        };
                        debug!("Connection from {}", &remote);
                        self.register_connection(&poll, stream)
                            .unwrap_or_else(|e| error!("{}", e));
                    }

                    Token(conn_id) => {
                        self.http_handler(conn_id, event, &poll, &mut response)
                            .unwrap_or_else(|e| error!("{}", e));
                    }
                }
            }
        }
    }

    fn register_connection(
        &mut self,
        poll: &Poll,
        stream: TcpStream,
    ) -> Result<(), failure::Error> {
        let token = Token(self.next_connection_id);
        poll.register(&stream, token, Ready::readable(), PollOpt::edge())?;

        if self
            .connections
            .insert(self.next_connection_id, stream)
            .is_some()
        {
            error!("Connection ID is already in use");
        }
        self.next_connection_id += 1;
        Ok(())
    }

    fn http_handler(
        &mut self,
        conn_id: usize,
        event: Event,
        poll: &Poll,
        response: &mut Vec<u8>,
    ) -> Result<(), failure::Error> {
        if event.readiness().is_readable() {
            self.read_from_socket(conn_id, poll, response)
        } else if event.readiness().is_writable() {
            self.write_to_socket(conn_id, response)
        } else {
            Err(failure::err_msg("Undefined event."))
        }
    }

    fn read_from_socket(
        &mut self,
        conn_id: usize,
        poll: &Poll,
        response: &mut Vec<u8>,
    ) -> Result<(), failure::Error> {
        debug!("readable conn_id: {}", conn_id);
        let mut buffer = [0u8; 1024];
        let stream = self.get_stream(conn_id)?;
        let nbytes = stream.read(&mut buffer)?;
        if nbytes != 0 {
            *response = make_response(&buffer[..nbytes])?;
            poll.reregister(stream, Token(conn_id), Ready::writable(), PollOpt::edge())?;
        } else {
            self.connections.remove(&conn_id);
        }

        Ok(())
    }

    fn write_to_socket(
        &mut self,
        conn_id: usize,
        response: &mut Vec<u8>,
    ) -> Result<(), failure::Error> {
        debug!("writable conn_id: {}", conn_id);
        let stream = self.get_stream(conn_id)?;
        stream.write_all(response)?;
        self.connections.remove(&conn_id);
        Ok(())
    }

    fn get_stream(&mut self, conn_id: usize) -> Result<&mut TcpStream, failure::Error> {
        self.connections
            .get_mut(&conn_id)
            .ok_or_else(|| failure::err_msg("Failed to get connection."))
    }
}

fn make_response(buffer: &[u8]) -> Result<Vec<u8>, failure::Error> {
    let http_pattern = Regex::new(r"(.*) (.*) HTTP/1.([0-1])\r\n.*")?;
    let captures = match http_pattern.captures(str::from_utf8(buffer)?) {
        Some(cap) => cap,
        None => {
            return create_msg_from_code(400, None);
        }
    };

    let method = captures[1].to_string();
    let path = format!(
        "{}{}{}",
        env::current_dir()?.display(),
        WEBROOT,
        &captures[2]
    );
    let _version = captures[3].to_string();

    if method != "GET" {
        return create_msg_from_code(501, None);
    }

    let file = match File::open(path) {
        Ok(file) => file,
        Err(_) => {
            return create_msg_from_code(404, None);
        }
    };
    let mut reader = BufReader::new(file);
    let mut buf = Vec::new();
    reader.read_to_end(&mut buf)?;
    create_msg_from_code(200, Some(buf))
}

const OK_200: &str = "200 OK";
const ERROR_400: &str = "400 Bad Request";
const ERROR_404: &str = "404 Not Found";
const ERROR_501: &str = "501 Not Implemented";

fn create_msg_from_code(status_code: u16, msg: Option<Vec<u8>>) -> Result<Vec<u8>, failure::Error> {
    match status_code {
        200 => {
            let header = create_res_msg(OK_200);
            let body = msg.unwrap_or_default();
            Ok([&header[..], &body[..]].concat())
        },
        400 => Ok(create_res_msg(ERROR_400)),
        404 => Ok(create_res_msg(ERROR_404)),
        501 => Ok(create_res_msg(ERROR_501)),
        _ => Err(failure::err_msg("Unexpected status code."))
    }
}

fn create_res_msg(msg: &str) -> Vec<u8> {
    format!("HTTP/1.0 {}\r\n\
             Server: mio webserver\r\n\r\n", msg)
             .to_string()
             .into_bytes()
}

fn main() {}
