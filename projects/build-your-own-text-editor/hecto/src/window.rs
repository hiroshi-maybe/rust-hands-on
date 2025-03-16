use std::{
    io::Read,
    os::{
        fd::RawFd,
        raw::{c_int, c_ulong, c_ushort},
    },
};

use crate::stdio::BufferedCommands;

#[repr(C)]
#[derive(Debug)]
struct Winsize {
    ws_row: c_ushort,
    ws_col: c_ushort,
    ws_xpixel: c_ushort,
    ws_ypixel: c_ushort,
}

const STDOUT_FILENO: RawFd = 1;
const TIOCGWINSZ: c_ulong = 0x40087468;

extern "C" {
    fn ioctl(fd: c_int, request: c_ulong, ...) -> c_int;
}

pub fn get_window_size() -> Result<(usize, usize), std::io::Error> {
    unsafe {
        let mut winsize = std::mem::MaybeUninit::<Winsize>::uninit();

        if ioctl(STDOUT_FILENO, TIOCGWINSZ, winsize.as_mut_ptr()) == -1 {
            let move_curosor_bottom_right_cmd = b"\x1b[999C\x1b[999B";
            let mut commands = BufferedCommands::new();
            commands.append(move_curosor_bottom_right_cmd);
            return get_cursor_position(&mut commands);
        }

        let winsize = winsize.assume_init();
        Ok((winsize.ws_row as usize, winsize.ws_col as usize))
    }
}

fn get_cursor_position(commands: &mut BufferedCommands) -> Result<(usize, usize), std::io::Error> {
    let query_term_status_info_cmd = b"\x1b[6n";
    commands.append(query_term_status_info_cmd);
    commands.execute()?;

    print!("\r\n");
    let stdin = std::io::stdin();
    let mut handle = stdin.lock();
    let mut buffer = [0; 1];
    let mut st = 0; // 0: reading the escape sequence, 1: reading the row, 2: reading the column
    let mut rows = 0;
    let mut cols = 0;
    while handle.read(&mut buffer).is_ok_and(|n| n == 1) {
        let c = buffer[0] as char;
        match c {
            '[' => {
                st = 1;
            }
            ';' => {
                st = 2;
            }
            '0'..='9' => match st {
                1 => {
                    rows = rows * 10 + (buffer[0] - '0' as u8) as usize;
                }
                2 => {
                    cols = cols * 10 + (buffer[0] - '0' as u8) as usize;
                }
                _ => {}
            },
            'R' => {
                break;
            }
            _ => {}
        }
    }

    Ok((rows, cols))
}
