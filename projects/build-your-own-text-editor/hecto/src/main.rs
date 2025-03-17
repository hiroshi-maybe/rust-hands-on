use std::io::Read;

use hecto::stdio::BufferedCommands;
use hecto::termios::enable_raw_mode;
use hecto::window::get_window_size;

fn main() {
    enable_raw_mode().expect("failed to enable raw mode");
    let config = EditorConfig::new().expect("failed to initialize editor config");

    refresh_screen(&config).expect("failed to refresh screen");
    let version = env!("CARGO_PKG_VERSION");

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
    let c: char = read_key();
    if c == '\0' {
        return false;
    }

    if c.is_ascii_control() {
        print!("{}\r\n", c as u8);
    } else {
        print!("{} ('{}')\r\n", c as u8, c as char);
    }

    match c {
        c if c == ctrl_key('q') => refresh_screen(config).is_ok(),
        _ => false,
    }
}

fn ctrl_key(c: char) -> char {
    (c as u8 & 0x1f) as char
}

// endregion: input

// region: output

fn refresh_screen(config: &EditorConfig) -> Result<(), std::io::Error> {
    let mut commmands = BufferedCommands::new();
    let make_cursor_invisible_cmd = b"\x1b[?25l";
    commmands.append(make_cursor_invisible_cmd);
    let reposition_cursor_cmd = b"\x1b[H";
    commmands.append(reposition_cursor_cmd);
    let make_cursor_visible_cmd = b"\x1b[?25h";
    commmands.append(make_cursor_visible_cmd);

    draw_rows(config, &mut commmands);

    commmands.append(reposition_cursor_cmd);
    commmands.execute()?;

    Ok(())
}

fn draw_rows(config: &EditorConfig, commands: &mut BufferedCommands) {
    let placeholder_tilde_line = b"~";
    let clear_line_cmd = b"\x1b[K";
    for i in 0..config.screen_rows {
        if i == config.screen_rows / 3 {
            let greeting = "Kilo editor -- version ".to_string() + env!("CARGO_PKG_VERSION");
            let mut padding = (config.screen_cols - greeting.len()) / 2;
            if padding > 0 {
                commands.append(placeholder_tilde_line);
                padding -= 1;
            }
            for _ in 0..padding {
                commands.append(&[b' ']);
            }
            commands.append(greeting.bytes().collect::<Vec<_>>().as_slice());
        } else {
            commands.append(placeholder_tilde_line);
        }
        commands.append(clear_line_cmd);
        if i < config.screen_rows - 1 {
            commands.append(b"\r\n");
        }
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
