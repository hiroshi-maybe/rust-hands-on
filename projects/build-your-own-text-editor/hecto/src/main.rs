use std::io::Read;

use hecto::termios::enable_raw_mode;

fn main() {
    enable_raw_mode().expect("failed to enable raw mode");

    let mut buffer = [0; 1];
    let stdin = std::io::stdin();
    let mut handle = stdin.lock();

    while handle.read(&mut buffer).is_ok_and(|n| n == 1) && buffer[0] != b'q' {}
}
