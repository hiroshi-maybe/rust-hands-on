use std::io::Read;

fn main() {
    let mut buffer = [0; 1];
    let stdin = std::io::stdin();
    let mut handle = stdin.lock();

    while handle.read(&mut buffer).is_ok_and(|n| n == 1) && buffer[0] != b'q' {}
}
