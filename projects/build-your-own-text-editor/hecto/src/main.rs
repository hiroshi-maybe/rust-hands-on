use std::io::Read;

use hecto::stdio::write_command;
use hecto::termios::enable_raw_mode;
use hecto::window::get_window_size;

fn main() {
    enable_raw_mode().expect("failed to enable raw mode");
    let config = EditorConfig::new().expect("failed to initialize editor config");

    dbg!(config.screen_cols, config.screen_rows);
    refresh_screen(&config);
    loop {
        if process_keypress(&config) {
            break;
        }
    }
}

// region: data

struct EditorConfig {
    screen_rows: usize,
    screen_cols: usize,
}

impl EditorConfig {
    fn new() -> Result<Self, std::io::Error> {
        let (screen_rows, screen_cols) = get_window_size()?;
        Ok(Self {
            screen_rows,
            screen_cols,
        })
    }
}

// endregion: data

// region: input

fn process_keypress(config: &EditorConfig) -> bool {
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
            refresh_screen(config);
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

fn refresh_screen(config: &EditorConfig) {
    let clear_screen_cmd = b"\x1b[2J";
    write_command(clear_screen_cmd);
    let reposition_cursor_cmd = b"\x1b[H";
    write_command(reposition_cursor_cmd);

    draw_rows(config);

    write_command(reposition_cursor_cmd);
}

fn draw_rows(config: &EditorConfig) {
    let placeholder_tilde_line = b"~\r\n";
    for _ in 0..config.screen_rows {
        write_command(placeholder_tilde_line);
    }
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
