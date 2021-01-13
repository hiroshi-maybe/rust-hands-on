use hello::ThreadPool;
use std::fs;
use std::io::prelude::*;
use std::net::TcpListener;
use std::net::TcpStream;
use std::thread;
use std::time::Duration;

// Hit http://127.0.0.1:7878/ in a browser

fn main() {
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();

    for stream in listener.incoming() {
        let stream = stream.unwrap();
        let pool = ThreadPool::new(4);

        pool.execute(|| {
            handle_connection(stream);
        });
    }
}

fn handle_connection(mut stream: TcpStream) {
    let mut buffer = [0; 1024];
    stream.read(&mut buffer).unwrap();

    let get = b"GET / HTTP/1.1\r\n";
    let sleep = b"GET /sleep HTTP/1.1\r\n";

    println!("Request: {}", String::from_utf8_lossy((&buffer[..])));

    if buffer.starts_with(get) {
        let response = response_from_file(200, "OK", "hello.html");
        write(stream, &response);
    } else if buffer.starts_with(sleep) {
        thread::sleep(Duration::from_secs(5));
        let response = response_from_file(200, "OK", "hello.html");
        write(stream, &response);
    } else {
        println!("Unexpected request");
        let response = response_from_file(404, "NOT FOUND", "404.html");
        write(stream, &response);
    }
}

fn response_from_file(status_code: u32, reason: &str, file_name: &str) -> String {
    let contents = fs::read_to_string(file_name).unwrap();
    format!(
        "HTTP/1.1 {} {}\r\nContent-Length: {}\r\n\r\n{}",
        status_code,
        reason,
        contents.len(),
        contents
    )
}

fn write(mut stream: TcpStream, response: &String) {
    stream.write(response.as_bytes()).unwrap();
    stream.flush().unwrap();
}
