use std::io::Read;

use hecto::termios::enable_raw_mode;

fn main() {
    enable_raw_mode().expect("failed to enable raw mode");

    let mut buffer = [0; 1];
    let stdin = std::io::stdin();
    let mut handle = stdin.lock();

    loop {
        buffer[0] = '\0' as u8;
        let _ = handle.read(&mut buffer).is_ok_and(|n| n == 1);

        let c = buffer[0] as char;
        if c == '\0' {
            continue;
        }

        if c.is_ascii_control() {
            print!("{}\r\n", c as u8);
        } else {
            print!("{} ('{}')\r\n", c as u8, c as char);
        }

        if c == 'q' {
            break;
        }
    }
}
