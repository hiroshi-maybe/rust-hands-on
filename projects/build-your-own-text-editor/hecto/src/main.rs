use std::io::Read;

use hecto::stdio::BufferedCommands;
use hecto::termios::enable_raw_mode;
use hecto::window::get_window_size;

fn main() {
    enable_raw_mode().expect("failed to enable raw mode");
    let mut config = EditorConfig::new().expect("failed to initialize editor config");

    loop {
        refresh_screen(&config).expect("failed to refresh screen");
        if process_keypress(&mut config) {
            break;
        }
    }
}

// region: data

struct EditorConfig {
    cx: usize,
    cy: usize,
    screen_rows: usize,
    screen_cols: usize,
}

impl EditorConfig {
    fn new() -> Result<Self, std::io::Error> {
        let (screen_rows, screen_cols) = get_window_size()?;
        Ok(Self {
            cx: 0,
            cy: 0,
            screen_rows,
            screen_cols,
        })
    }
}

// endregion: data

// region: input

fn move_cursor(config: &mut EditorConfig, c: char) {
    match c {
        'a' if config.cx > 0 => {
            config.cx -= 1;
        }
        'd' if config.cx < config.screen_cols - 1 => {
            config.cx += 1;
        }
        'w' if config.cy < config.screen_rows - 1 => {
            config.cy += 1;
        }
        's' if config.cy > 0 => {
            config.cy -= 1;
        }
        _ => {}
    }
}

fn process_keypress(config: &mut EditorConfig) -> bool {
    let c: char = read_key();
    if c == '\0' {
        return false;
    }

    match c {
        c if c == ctrl_key('q') => {
            return refresh_screen(config).is_ok();
        }
        'a' | 'd' | 'w' | 's' => move_cursor(config, c),
        _ => {}
    }

    false
}

fn ctrl_key(c: char) -> char {
    (c as u8 & 0x1f) as char
}

// endregion: input

// region: output

fn refresh_screen(config: &EditorConfig) -> Result<(), std::io::Error> {
    let make_cursor_invisible_cmd = b"\x1b[?25l";
    let reposition_cursor_cmd = b"\x1b[H";
    let mut commmands = BufferedCommands::new(
        [
            make_cursor_invisible_cmd.as_slice(),
            reposition_cursor_cmd.as_slice(),
        ]
        .concat(),
    );

    draw_rows(config, &mut commmands);

    let place_cursor_cmd = format!("\x1b[{};{}H", config.cy + 1, config.cx + 1);
    commmands.append(place_cursor_cmd.as_bytes());
    let make_cursor_visible_cmd = b"\x1b[?25h";
    commmands.append(make_cursor_visible_cmd);
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
