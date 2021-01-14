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

    // Threads are spawned and one of them takes and keeps a lock.
    let pool = ThreadPool::new(4);

    for stream in listener.incoming() {
        let stream = stream.unwrap();

        println!("NEW STREAM");

        pool.execute(|id| {
            println!("[Worker {}] start a job", id);
            handle_connection(stream, id);
            println!("[Worker {}] finish a job", id);
        });
    }
}

fn handle_connection(mut stream: TcpStream, id: usize) {
    let mut buffer = [0; 1024];
    println!("[Worker {}] start reading a stream", id);
    stream.read(&mut buffer).unwrap();
    println!("[Worker {}] finish reading a stream", id);

    let get = b"GET / HTTP/1.1\r\n";
    let sleep = b"GET /sleep HTTP/1.1\r\n";

    let rawReq = String::from_utf8_lossy((&buffer[..]));

    if buffer.starts_with(get) {
        let response = response_from_file(200, "OK", "hello.html");
        write(stream, &response);
    } else if buffer.starts_with(sleep) {
        thread::sleep(Duration::from_secs(5));
        let response = response_from_file(200, "OK", "hello.html");
        write(stream, &response);
    } else {
        println!("Unexpected request");
        println!("Request: {}", rawReq);
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
