use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Read, Write};

use kilo_rs::file::truncate_file;
use kilo_rs::stdio::BufferedCommands;
use kilo_rs::termios::enable_raw_mode;
use kilo_rs::window::get_window_size;

fn main() {
    enable_raw_mode().expect("failed to enable raw mode");
    let mut config = EditorConfig::new().expect("failed to initialize editor config");

    let args = std::env::args();
    if args.len() > 1 {
        let file_name = args.skip(1).next().expect("failed to get file name");
        editor_open(file_name.as_str(), &mut config).expect("failed to open file");
    }

    set_status_message(
        &mut config,
        "HELP: Ctrl-S = save | Ctrl-Q = quit | Ctrl-F = find",
    );

    loop {
        refresh_screen(&mut config).expect("failed to refresh screen");
        if process_keypress(&mut config) {
            break;
        }
    }
}

// region: defines

const TAB_STOP: usize = 8;
const KILO_QUIT_TIMES: usize = 3;

const CR: char = '\r';
const LF: char = '\n';
const CTRL_F: char = ctrl_key('f');
const CTRL_Q: char = ctrl_key('q');
const CTRL_S: char = ctrl_key('s');
const CTRL_L: char = ctrl_key('l');
const ESCAPE: char = '\x1b';
const CTRL_H: char = ctrl_key('h');
const BACKSPACE: char = '\x7f';

struct EditorRow {
    chars: Vec<char>,
    render: Vec<char>,
}

impl EditorRow {
    fn new(chars: Vec<char>) -> Self {
        Self {
            chars,
            render: vec![],
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum EditorKey {
    Backspace,
    Arrow(ArrowDirection),
    Page(PageDirection),
    Home,
    End,
    Del,
    Char(char),
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum ArrowDirection {
    Left,
    Right,
    Up,
    Down,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum PageDirection {
    Up,
    Down,
}

// endregion: defines

// region: data

struct EditorConfig {
    cx: usize,
    cy: usize,
    rx: usize,
    row_offset: usize,
    col_offset: usize,
    rows: Vec<EditorRow>,
    dirty: bool,
    quit_times: usize,
    file_name: Option<String>,
    status_msg: Option<String>,
    status_msg_time: std::time::Instant,
    screen_rows: usize,
    screen_cols: usize,
    last_match: Option<usize>,
    match_direction: bool,
}

impl EditorConfig {
    fn new() -> Result<Self, std::io::Error> {
        let (screen_rows, screen_cols) = get_window_size()?;
        Ok(Self {
            cx: 0,
            cy: 0,
            rx: 0,
            row_offset: 0,
            col_offset: 0,
            rows: vec![],
            dirty: false,
            quit_times: KILO_QUIT_TIMES,
            file_name: None,
            status_msg: None,
            status_msg_time: std::time::Instant::now(),
            screen_rows: screen_rows - 2,
            screen_cols,
            last_match: None,
            match_direction: true,
        })
    }
}

// endregion: data

// region: input

fn move_cursor(config: &mut EditorConfig, dir: ArrowDirection) {
    let col_limit = config
        .rows
        .get(config.cy)
        .map(|r| r.chars.len())
        .unwrap_or(0);
    match dir {
        ArrowDirection::Left => {
            if config.cx > 0 {
                config.cx -= 1;
            } else if config.cy > 0 {
                config.cy -= 1;
                config.cx = config.rows[config.cy].chars.len();
            }
        }
        ArrowDirection::Right => {
            if config.cx < col_limit {
                config.cx += 1;
            } else if config.cy < config.rows.len() && config.cx == col_limit {
                config.cy += 1;
                config.cx = 0;
            }
        }
        ArrowDirection::Down if config.cy < config.rows.len() => {
            config.cy += 1;
        }
        ArrowDirection::Up if config.cy > 0 => {
            config.cy -= 1;
        }
        _ => {}
    }

    config.cx = if let Some(row) = config.rows.get(config.cy) {
        config.cx.min(row.chars.len())
    } else {
        0
    }
}

fn process_keypress(config: &mut EditorConfig) -> bool {
    let c = read_key();
    // dbg!(c.clone());
    match c {
        EditorKey::Char(c) => match c {
            CTRL_Q => {
                if config.dirty && config.quit_times > 0 {
                    set_status_message(
                        config,
                        format!("WARNING!!! File has unsaved changes. Press Ctrl-Q {} more times to quit.", config.quit_times)
                            .as_str(),
                    );
                    config.quit_times -= 1;
                    return false;
                }
                return refresh_screen(config).is_ok();
            }
            CTRL_S => {
                _ = editor_save(config).inspect_err(|e| {
                    set_status_message(config, format!("Can't save! I/O error: {}", e).as_str());
                });
            }
            CTRL_F => {
                editor_find(config);
            }
            CR => {
                editor_insert_new_line(config);
            }
            ESCAPE | CTRL_L => {
                // Ignore escape key
            }
            _ => editor_insert_char(config, c),
        },
        EditorKey::Backspace | EditorKey::Del => {
            if c == EditorKey::Del {
                move_cursor(config, ArrowDirection::Right);
            }
            editor_del_char(config);
        }
        EditorKey::Arrow(dir) => move_cursor(config, dir),
        EditorKey::Page(dir) => {
            let (key, adjusted_cy) = match dir {
                PageDirection::Up => (ArrowDirection::Up, config.row_offset),
                PageDirection::Down => (
                    ArrowDirection::Down,
                    config
                        .rows
                        .len()
                        .min(config.row_offset + config.screen_rows - 1),
                ),
            };
            config.cy = adjusted_cy;
            for _ in 0..config.screen_rows {
                move_cursor(config, key);
            }
        }
        EditorKey::Home => config.cx = 0,
        EditorKey::End => {
            config.cx = if let Some(row) = config.rows.get(config.cy) {
                row.chars.len()
            } else {
                0
            }
        }
    }

    config.quit_times = KILO_QUIT_TIMES;

    false
}

fn editor_prompt<F, C>(formatter: F, callback: C, config: &mut EditorConfig) -> Option<String>
where
    F: Fn(&str) -> String,
    C: Fn(&str, EditorKey, &mut EditorConfig),
{
    let mut buf = vec![];
    loop {
        set_status_message(
            config,
            formatter(buf.iter().collect::<String>().as_str()).as_str(),
        );
        refresh_screen(config).ok()?;
        let key = read_key();
        match key {
            EditorKey::Backspace => {
                buf.pop();
            }
            EditorKey::Char(c) => match c {
                ESCAPE => {
                    set_status_message(config, "");
                    callback(buf.iter().collect::<String>().as_str(), key, config);
                    return None;
                }
                CR if !buf.is_empty() => {
                    set_status_message(config, "");
                    callback(buf.iter().collect::<String>().as_str(), key, config);
                    break;
                }
                c if !c.is_ascii_control() && (c as u8) < 128 => {
                    buf.push(c);
                }
                _ => {}
            },
            _ => {}
        }
        callback(buf.iter().collect::<String>().as_str(), key, config);
    }

    Some(buf.iter().collect())
}

const fn ctrl_key(c: char) -> char {
    (c as u8 & 0x1f) as char
}

// endregion: input

// region: output

fn refresh_screen(config: &mut EditorConfig) -> Result<(), std::io::Error> {
    editor_scroll(config);
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
    draw_status_bar(config, &mut commmands);
    draw_message_bar(config, &mut commmands);

    let place_cursor_cmd = format!(
        "\x1b[{};{}H",
        config.cy - config.row_offset + 1,
        config.rx - config.col_offset + 1
    );
    commmands.append(place_cursor_cmd.as_bytes());
    let make_cursor_visible_cmd = b"\x1b[?25h";
    commmands.append(make_cursor_visible_cmd);
    commmands.execute()?;

    Ok(())
}

fn editor_scroll(config: &mut EditorConfig) {
    config.rx = config.cx;
    if config.cy < config.rows.len() {
        config.rx = map_row_cx_to_rx(&config.rows[config.cy], config.cx);
    }

    config.row_offset = config.row_offset.min(config.cy);
    if config.cy >= config.row_offset + config.screen_rows {
        config.row_offset = config.cy - config.screen_rows + 1;
    }
    config.col_offset = config.col_offset.min(config.rx);
    if config.rx >= config.col_offset + config.screen_cols {
        config.col_offset = config.rx - config.screen_cols + 1;
    }
}

fn draw_rows(config: &EditorConfig, commands: &mut BufferedCommands) {
    dbg!(config.cx, config.col_offset, config.screen_cols);
    dbg!(config.cy, config.row_offset, config.screen_rows);
    for y in 0..config.screen_rows {
        let file_row = y + config.row_offset;
        if file_row >= config.rows.len() {
            if config.rows.len() == 0 && y == config.screen_rows / 3 {
                draw_welcome_greeting(config, commands);
            } else {
                let placeholder_tilde_line = b"~";
                commands.append(placeholder_tilde_line);
            }
        } else {
            let len = config.rows[file_row]
                .render
                .len()
                .checked_sub(config.col_offset)
                .unwrap_or(0);
            let len = len.min(config.screen_cols);
            if config.col_offset < config.rows[file_row].render.len() {
                let line =
                    &config.rows[file_row].render[config.col_offset..config.col_offset + len];
                commands.append(line.iter().collect::<String>().as_bytes());
            } else {
                commands.append(b"");
            };
        }

        let clear_line_cmd = b"\x1b[K";
        commands.append(clear_line_cmd);
        commands.append(b"\r\n");
    }
}

fn draw_welcome_greeting(config: &EditorConfig, commands: &mut BufferedCommands) {
    let greeting = "Kilo editor -- version ".to_string() + env!("CARGO_PKG_VERSION");
    let mut padding = (config.screen_cols - greeting.len()) / 2;
    if padding > 0 {
        let placeholder_tilde_line = b"~";
        commands.append(placeholder_tilde_line);
        padding -= 1;
    }
    for _ in 0..padding {
        commands.append(&[b' ']);
    }
    commands.append(greeting.bytes().collect::<Vec<_>>().as_slice());
}

fn draw_status_bar(config: &EditorConfig, commands: &mut BufferedCommands) {
    let file_name = config.file_name.as_deref().unwrap_or("[No Name]");
    let lines = config.rows.len();
    let status_left = format!(
        "{:.20} - {} lines {}",
        file_name,
        lines,
        if config.dirty { "(modified)" } else { "" }
    );
    let status_right = format!("{}/{}", config.cy + 1, config.rows.len());
    draw_text_in_status_bar(config, &status_left, &status_right, commands);
    commands.append(b"\r\n");
}

fn draw_text_in_status_bar(
    config: &EditorConfig,
    text_left: &str,
    text_right: &str,
    commands: &mut BufferedCommands,
) {
    let inverted_color_cmd = b"\x1b[7m";
    commands.append(inverted_color_cmd);
    let mut len = text_left.as_bytes().len().min(config.screen_cols);
    commands.append(&text_left.as_bytes()[..len]);
    while len < config.screen_cols {
        if len + text_right.as_bytes().len() == config.screen_cols {
            commands.append(text_right.as_bytes());
            break;
        }
        commands.append(&[b' ']);
        len += 1;
    }

    let clear_text_attributes_cmd = b"\x1b[m";
    commands.append(clear_text_attributes_cmd);
}

fn draw_message_bar(config: &EditorConfig, commands: &mut BufferedCommands) {
    commands.append(b"\x1b[K");
    let msg = config.status_msg.as_deref().unwrap_or("");
    let msg_len = msg.len().min(config.screen_cols);
    let msg = &msg[..msg_len];

    if msg.len() > 0 && config.status_msg_time.elapsed().as_secs() < 5 {
        commands.append(msg.as_bytes());
    }
}

fn set_status_message(config: &mut EditorConfig, msg: &str) {
    config.status_msg = Some(msg.to_string());
    config.status_msg_time = std::time::Instant::now();
}

// endregion: output

// region: file i/o

fn editor_open(file_name: &str, config: &mut EditorConfig) -> std::io::Result<()> {
    let file = File::open(file_name).expect("failed to open file");
    config.file_name = Some(file_name.to_string());
    let reader = BufReader::new(file);
    for line in reader.lines() {
        let line = line?;
        editor_insert_row(config.rows.len(), line, config);
    }
    config.dirty = false;

    Ok(())
}

fn editor_save(config: &mut EditorConfig) -> std::io::Result<()> {
    let Some(file_name) = config.file_name.clone().or_else(|| {
        editor_prompt(
            |file_name| format!("Save as: {}", file_name),
            |_, _, _| (),
            config,
        )
    }) else {
        set_status_message(config, "Save aborted");
        return Ok(());
    };

    let content = editor_rows_to_string(config);
    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(file_name)?;

    truncate_file(&mut file, content.len())?;
    let dat = content.as_bytes();
    file.write_all(dat)?;
    config.dirty = false;
    set_status_message(
        config,
        format!("{} bytes written to disk", dat.len()).as_str(),
    );

    Ok(())
}

fn editor_rows_to_string(config: &EditorConfig) -> String {
    let mut content = String::new();
    for row in &config.rows {
        content.push_str(&row.chars.iter().collect::<String>());
        content.push(LF);
    }

    content
}

// endregion: file i/o

// region: find

fn editor_find_callback(query: &str, key: EditorKey, config: &mut EditorConfig) {
    if key == EditorKey::Char(ESCAPE) || key == EditorKey::Char(CR) {
        config.last_match = None;
        config.match_direction = true;
        return;
    }

    match key {
        EditorKey::Arrow(ArrowDirection::Down) | EditorKey::Arrow(ArrowDirection::Right) => {
            config.match_direction = true;
        }
        EditorKey::Arrow(ArrowDirection::Up) | EditorKey::Arrow(ArrowDirection::Left) => {
            config.match_direction = false;
        }
        _ => {
            config.last_match = None;
            config.match_direction = true;
        }
    }

    if config.last_match.is_none() {
        config.match_direction = true;
    }

    let delta = if config.match_direction { 1isize } else { -1 };
    let cur = mod_add(
        config.last_match.unwrap_or(config.rows.len() - 1),
        delta,
        config.rows.len(),
    );
    let query = query.chars().collect::<Vec<_>>();
    let Some((i, j)) = (0..config.rows.len()).find_map(|offset| {
        let i = mod_add(cur, offset as isize * delta, config.rows.len());
        let row = &config.rows[i];
        if let Some(j) = row
            .chars
            .windows(query.len())
            .position(|w| w == query.as_slice())
        {
            Some((i, j))
        } else {
            None
        }
    }) else {
        return;
    };

    // matched
    config.last_match = Some(i);
    config.cy = i;
    config.cx = j;
    config.row_offset = config.rows.len();
}

fn mod_add(a: usize, b: isize, modulus: usize) -> usize {
    (a as isize + modulus as isize + b) as usize % modulus
}

fn editor_find(config: &mut EditorConfig) {
    let original_cx = config.cx;
    let original_cy = config.cy;
    let original_col_offset = config.col_offset;
    let original_row_offset = config.row_offset;

    if editor_prompt(
        |query| format!("Search: {} (Use ESC/Arrows/Enter)", query),
        |query, key, config| editor_find_callback(query, key, config),
        config,
    )
    .is_none()
    {
        config.cx = original_cx;
        config.cy = original_cy;
        config.col_offset = original_col_offset;
        config.row_offset = original_row_offset;
    }
}

// endregion: find

// region: row operations

fn map_row_cx_to_rx(row: &EditorRow, cx: usize) -> usize {
    let mut rx = 0;
    for i in 0..cx {
        if row.chars[i] == '\t' {
            rx = rx + TAB_STOP - (rx % TAB_STOP);
        } else {
            rx += 1;
        }
    }

    rx
}

fn update_row(row: &mut EditorRow) {
    row.render.clear();
    for i in 0..row.chars.len() {
        if row.chars[i] == '\t' {
            while row.render.len() % TAB_STOP != 0 {
                row.render.push(' ');
            }
        } else {
            row.render.push(row.chars[i]);
        }
    }
}

fn editor_insert_row(at: usize, line: String, config: &mut EditorConfig) {
    if at > config.rows.len() {
        return;
    }

    let mut row = EditorRow::new(line.trim_end().chars().collect());
    update_row(&mut row);
    config.rows.insert(at, row);
    config.dirty = true;
}

fn editor_del_row(config: &mut EditorConfig, at: usize) -> Option<EditorRow> {
    if at >= config.rows.len() {
        return None;
    }

    let row = config.rows.remove(at);

    config.dirty = true;

    Some(row)
}

fn row_insert_char(row: &mut EditorRow, at: usize, c: char, dirty: &mut bool) {
    let at = at.min(row.chars.len());
    row.chars.insert(at, c);
    update_row(row);
    *dirty = true;
}

fn editor_row_append_string(row: &mut EditorRow, s: &mut Vec<char>, dirty: &mut bool) {
    row.chars.append(s);
    update_row(row);
    *dirty = true;
}

fn row_del_char(row: &mut EditorRow, at: usize, dirty: &mut bool) {
    if at >= row.chars.len() {
        return;
    }

    row.chars.remove(at);
    update_row(row);
    *dirty = true;
}

// endregion: row operations

// region: editor operations

fn editor_insert_char(config: &mut EditorConfig, c: char) {
    if config.cy == config.rows.len() {
        editor_insert_row(config.rows.len(), "".to_string(), config);
    }
    row_insert_char(&mut config.rows[config.cy], config.cx, c, &mut config.dirty);
    config.cx += 1;
}

fn editor_insert_new_line(config: &mut EditorConfig) {
    if config.cx == 0 {
        editor_insert_row(config.cy, "".to_string(), config);
    } else {
        let row = &mut config.rows[config.cy];
        let new_row = row.chars.split_off(config.cx);
        update_row(row);
        editor_insert_row(config.cy + 1, new_row.iter().collect(), config);
    }
    config.cy += 1;
    config.cx = 0;
}

fn editor_del_char(config: &mut EditorConfig) {
    if config.cy == config.rows.len() {
        return;
    }
    if config.cx == 0 && config.cy == 0 {
        return;
    }

    if config.cx > 0 {
        let row = &mut config.rows[config.cy];
        row_del_char(row, config.cx - 1, &mut config.dirty);
        config.cx -= 1;
    } else {
        config.cx = config.rows[config.cy - 1].chars.len();
        let Some(mut row) = editor_del_row(config, config.cy) else {
            return;
        };

        editor_row_append_string(
            &mut config.rows[config.cy - 1],
            &mut row.chars,
            &mut config.dirty,
        );
        config.cy -= 1;
    }
}

// endregion: editor operations

// region: terminal

fn read_key() -> EditorKey {
    let stdin = std::io::stdin();
    let mut handle = stdin.lock();
    let mut buffer = [0; 1];
    buffer[0] = '\0' as u8;
    while !handle.read(&mut buffer).is_ok_and(|n| n == 1) {}
    let c = buffer[0] as char;

    // Escape sequence
    if c == ESCAPE {
        let mut seq = [0; 2];
        if handle.read(&mut seq).is_ok_and(|n| n == 2) {
            match seq[0] {
                b'[' => {
                    let mut seq2 = [0; 1];
                    match seq[1] {
                        b'A' => EditorKey::Arrow(ArrowDirection::Up),
                        b'B' => EditorKey::Arrow(ArrowDirection::Down),
                        b'C' => EditorKey::Arrow(ArrowDirection::Right),
                        b'D' => EditorKey::Arrow(ArrowDirection::Left),
                        b'H' => EditorKey::Home,
                        b'F' => EditorKey::End,
                        b'1' | b'3' | b'4' | b'5' | b'6' | b'7' | b'8'
                            if handle.read(&mut seq2).is_ok_and(|n| n == 1) && seq2[0] == b'~' =>
                        {
                            match seq[1] {
                                b'1' | b'7' => EditorKey::Home,
                                b'4' | b'8' => EditorKey::End,
                                b'3' => EditorKey::Del,
                                b'5' => EditorKey::Page(PageDirection::Up),
                                b'6' => EditorKey::Page(PageDirection::Down),
                                _ => unreachable!(),
                            }
                        }
                        _ => EditorKey::Char(ESCAPE),
                    }
                }
                b'O' => match seq[1] {
                    b'H' => EditorKey::Home,
                    b'F' => EditorKey::End,
                    _ => EditorKey::Char(ESCAPE),
                },
                _ => EditorKey::Char(ESCAPE),
            }
        } else {
            EditorKey::Char(ESCAPE)
        }
    } else {
        if c == CTRL_H || c == BACKSPACE {
            EditorKey::Backspace
        } else {
            EditorKey::Char(c)
        }
    }
}

// endregion: terminal
