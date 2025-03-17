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

// region: defines

#[derive(Debug)]
enum EditorKey {
    Arrow(ArrowDirection),
    Page(PageDirection),
    Char(char),
}

#[derive(Debug, Clone, Copy)]
enum ArrowDirection {
    Left,
    Right,
    Up,
    Down,
}

#[derive(Debug, Clone, Copy)]
enum PageDirection {
    Up,
    Down,
}

// endregion: defines

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

fn move_cursor(config: &mut EditorConfig, dir: ArrowDirection) {
    dbg!(dir);
    match dir {
        ArrowDirection::Left if config.cx > 0 => {
            config.cx -= 1;
        }
        ArrowDirection::Right if config.cx < config.screen_cols - 1 => {
            config.cx += 1;
        }
        ArrowDirection::Down if config.cy < config.screen_rows - 1 => {
            config.cy += 1;
        }
        ArrowDirection::Up if config.cy > 0 => {
            config.cy -= 1;
        }
        _ => {}
    }
}

fn process_keypress(config: &mut EditorConfig) -> bool {
    let c = read_key();
    match c {
        EditorKey::Char(c) if c == ctrl_key('q') => {
            return refresh_screen(config).is_ok();
        }
        EditorKey::Arrow(dir) => move_cursor(config, dir),
        EditorKey::Page(dir) => {
            let key = match dir {
                PageDirection::Up => ArrowDirection::Up,
                PageDirection::Down => ArrowDirection::Down,
            };
            for _ in 0..config.screen_rows {
                move_cursor(config, key);
            }
        }
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

fn read_key() -> EditorKey {
    let stdin = std::io::stdin();
    let mut handle = stdin.lock();
    let mut buffer = [0; 1];
    buffer[0] = '\0' as u8;
    while !handle.read(&mut buffer).is_ok_and(|n| n == 1) {}
    let c = buffer[0] as char;

    // Escape sequence
    if c == '\x1b' {
        let mut seq = [0; 2];
        if handle.read(&mut seq).is_ok_and(|n| n == 2) {
            if seq[0] == b'[' {
                let mut seq2 = [0; 1];
                match seq[1] {
                    b'A' => EditorKey::Arrow(ArrowDirection::Up),
                    b'B' => EditorKey::Arrow(ArrowDirection::Down),
                    b'C' => EditorKey::Arrow(ArrowDirection::Right),
                    b'D' => EditorKey::Arrow(ArrowDirection::Left),
                    b'5' | b'6'
                        if handle.read(&mut seq2).is_ok_and(|n| n == 1) && seq2[0] == b'~' =>
                    {
                        match seq[1] {
                            b'5' => EditorKey::Page(PageDirection::Up),
                            b'6' => EditorKey::Page(PageDirection::Down),
                            _ => unreachable!(),
                        }
                    }
                    _ => EditorKey::Char('\x1b'),
                }
            } else {
                EditorKey::Char('\x1b')
            }
        } else {
            EditorKey::Char('\x1b')
        }
    } else {
        EditorKey::Char(c)
    }
}

// endregion: terminal
