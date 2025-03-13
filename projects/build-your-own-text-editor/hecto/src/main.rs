use std::io::Read;

use hecto::stdio::write_command;
use hecto::termios::enable_raw_mode;

fn main() {
    enable_raw_mode().expect("failed to enable raw mode");

    refresh_screen();
    loop {
        if process_keypress() {
            break;
        }
    }
}

// region: input

fn process_keypress() -> bool {
    let c = read_key();
    if c == '\0' {
        return false;
    }

    if c.is_ascii_control() {
        print!("{}\r\n", c as u8);
    } else {
        print!("{} ('{}')\r\n", c as u8, c as char);
    }

    match c {
        c if c == ctrl_key('q') => {
            refresh_screen();
            return true;
        }
        _ => false,
    }
}

fn ctrl_key(c: char) -> char {
    (c as u8 & 0x1f) as char
}

// endregion: input

// region: output

fn refresh_screen() {
    let clear_screen_cmd = b"\x1b[2J";
    write_command(clear_screen_cmd);
    let reposition_cursor_cmd = b"\x1b[H";
    write_command(reposition_cursor_cmd);
}

// endregion: output

// region: terminal

fn read_key() -> char {
    let stdin = std::io::stdin();
    let mut handle = stdin.lock();
    let mut buffer = [0; 1];
    buffer[0] = '\0' as u8;
    while !handle.read(&mut buffer).is_ok_and(|n| n == 1) {}
    buffer[0] as char
}

// endregion: terminal
