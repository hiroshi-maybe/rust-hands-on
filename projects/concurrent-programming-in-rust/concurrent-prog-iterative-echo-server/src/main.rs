use std::{
    io::{BufRead, BufReader, BufWriter, Write},
    net::TcpListener,
};

/// $ socat stdio tcp:localhost:10000

fn main() {
    let listener = TcpListener::bind("127.0.0.1:10000").unwrap();

    while let Ok((stream, _)) = listener.accept() {
        let s = stream.try_clone().unwrap();
        let mut reader = BufReader::new(stream);
        let mut writer = BufWriter::new(s);

        let mut buf = String::new();
        reader.read_line(&mut buf).unwrap();
        writer.write(buf.as_bytes()).unwrap();
        writer.flush().unwrap();
    }
}
